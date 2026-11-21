//! The game-side glue: the colony registry, founding, the citizen's job and `/minecolonies`.
//!
//! In the SDK layout this is a plain module of the mod's own crate (no injection, no `support.rs`
//! header rule from the retired `.hnmod`); the pure rules stay in [`crate::runtime`], which this
//! file only calls.
//!
//! Where the colony lives: the retired bundle scanned `game.world.block_entities` for town halls.
//! The SDK has no enumeration of a world's block entities, but it does have **attachments** —
//! mod data on levels, chunks, entities, players and block entities — and an *attachment query*
//! that lists every holder of one key in the current dimension. So the colony is one attachment
//! per town hall block entity (`COLONY`), and the farmer's hut is one marker attachment per hut
//! (`HUT`). A colony therefore still belongs to its hall, still saves with the world (the codec is
//! the saved form), and is still found by walking the halls that are loaded.
//!
//! Read from MineColonies 1.1.1411 (`ColonyManager`, `CreateColonyMessage`, `ChunkDataHelper`,
//! `EntityAIWorkFarmer`, `JobFarmer`). See `../../FINDINGS.md`.

use std::sync::Mutex;

use mod_api::{
    AttachmentCodec, AttachmentError, AttachmentHolder, AttachmentQueries, AttachmentType,
    BlockKey, BlockUpdateFlags, Command, CommandArgumentValue, CommandSource, Compound, EntityRef,
    Flow, Hand, Holder, Host, HostTransaction, Identifier, ItemResource, ItemStack, LevelRef,
    MobRef, PlayerRef, RegisterCommandsEvent, RunModCommandEvent, ServerHost, SpawnReason, Tag,
    Text,
};

use crate::runtime::{
    AIR, CITIZEN, Citizen, Colony, FARMER_HUT, FARMER_REACH, Job, LUMBERJACK_HUT, LUMBERJACK_REACH,
    LUMBERJACK_SEARCH, MAX_COLONY_SIZE_CHUNKS, MINER_HUT, Owner, Pos, Request, Step, TOWN_HALL,
    TownHall, Value, WHEAT, WHEAT_MAX_AGE, WHEAT_REPLANT_AGE, clamp_size, colony_fields,
    colony_from_fields, colony_tag, default_name, farmer_step, founding_refusal, grows_a_sapling,
    hand_status, hire_refusal, horizontal_distance, is_hut, is_log, lumberjack_step, may_manage,
    miner_next, radius_blocks, read_colony_tag, requests_line, requests_status, sapling_for,
    should_work, tool_missing, tree_at,
};

/// The vanilla mob a citizen falls back to when the content type is not loaded.
const VANILLA_VILLAGER: &str = "minecraft:villager";
/// The names a hired citizen can get: our own list, not the mod's name files.
const NAMES: [&str; 8] = ["Ada", "Bo", "Cass", "Dee", "Eli", "Fay", "Gus", "Hana"];
/// The block a citizen's crop search looks at, horizontally.
const CROP_RANGE: i32 = 6;
/// Vertically.
const CROP_HEIGHT: i32 = 2;
/// How far around a worker its colony's chest is looked for, horizontally and vertically.
const STORAGE_SEARCH: i32 = 16;
const STORAGE_HEIGHT: i32 = 8;
/// The colony's store: a chest standing inside the colony keeps what its workers collect.
const CHEST: &str = "minecraft:chest";
/// How far above and below a worker its tree's logs are looked for.
const LOG_SCAN_HEIGHT: i32 = 16;
/// Why a hired citizen appears: a command summons it, as `/summon` does (not natural spawning).
const SUMMONED: SpawnReason = SpawnReason::Command;

// ---------------------------------------------------------------------------
// The colony registry: one attachment per town hall block entity
// ---------------------------------------------------------------------------

/// A town hall's colony, saved with the hall's block entity holding it.
static COLONY: AttachmentType<Colony> = AttachmentType {
    namespace: "minecolonies",
    name: "colony",
    default: placeholder_colony,
    codec: Some(AttachmentCodec {
        encode: encode_colony,
        decode: decode_colony,
    }),
    copy_on_death: false,
    sync: false,
};

/// A hut standing in a colony: which building it is, so one marker answers every job's rule.
static HUT: AttachmentType<HutMarker> = AttachmentType {
    namespace: "minecolonies",
    name: "hut",
    default: HutMarker::default,
    codec: Some(AttachmentCodec {
        encode: encode_hut,
        decode: decode_hut,
    }),
    copy_on_death: false,
    sync: false,
};

/// The hut marker's value: the block's id, which is the building the job's rule asks for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HutMarker {
    /// The hut block (`minecolonies:blockhutfarmer`, …).
    pub(crate) building: String,
}

/// The value an absent colony reads as. `COLONY` is only ever read with `get_existing`, so this
/// is never stored; it exists because `AttachmentType` asks for a default.
fn placeholder_colony() -> Colony {
    Colony {
        id: 0,
        name: String::new(),
        owner: String::new(),
        owner_name: String::new(),
        center: Pos::new(0, 0, 0),
        size_chunks: 0,
        founded: 0,
        harvested: 0,
        felled: 0,
        dug: 0,
        requests: Vec::new(),
        citizens: Vec::new(),
    }
}

/// The colony record as the saved compound, through the runtime's field mapping (the same
/// function the retired `.hnmod` wrote into the block entity's `Colony` tag).
fn encode_colony(colony: &Colony) -> Result<Compound, AttachmentError> {
    let mut saved = Compound::new();
    for (key, value) in colony_fields(colony) {
        let tag = match value {
            Value::Int(value) => Tag::Int(value),
            Value::Long(value) => Tag::Long(value),
            Value::Str(value) => Tag::String(value.as_str().into()),
            Value::Citizens(rows) => Tag::List(
                rows.into_iter()
                    .map(|(id, name, job)| {
                        Tag::compound([
                            ("Id", Tag::String(id.as_str().into())),
                            ("Name", Tag::String(name.as_str().into())),
                            ("Job", Tag::String(job.as_str().into())),
                        ])
                    })
                    .collect(),
            ),
            Value::Requests(rows) => Tag::List(
                rows.into_iter()
                    .map(|request| {
                        Tag::compound([
                            ("Citizen", Tag::String(request.citizen.as_str().into())),
                            ("Name", Tag::String(request.name.as_str().into())),
                            ("Job", Tag::String(request.job.id().into())),
                            ("Item", Tag::String(request.item.as_str().into())),
                        ])
                    })
                    .collect(),
            ),
        };
        saved.insert(key.into(), tag);
    }
    Ok(saved)
}

/// A saved compound back to the record, through the runtime's parser.
fn decode_colony(saved: &Compound) -> Result<Colony, AttachmentError> {
    let fields = read_fields(saved);
    colony_from_fields(&fields).map_err(|_| AttachmentError::Decode)
}

fn encode_hut(marker: &HutMarker) -> Result<Compound, AttachmentError> {
    let mut saved = Compound::new();
    saved.insert(
        "Building".into(),
        Tag::String(marker.building.as_str().into()),
    );
    Ok(saved)
}

fn decode_hut(saved: &Compound) -> Result<HutMarker, AttachmentError> {
    text_of(saved, "Building")
        .map(|building| HutMarker { building })
        .ok_or(AttachmentError::Decode)
}

fn read_fields(map: &Compound) -> Vec<(&'static str, Value)> {
    let mut fields = Vec::new();
    for key in [
        "Id",
        "Name",
        "Owner",
        "OwnerName",
        "Center",
        "Size",
        "Founded",
        "Harvested",
        "Felled",
        "Dug",
        "Citizens",
        "Requests",
    ] {
        let Some(tag) = map.get(&key.into()) else {
            continue;
        };
        let value = match (key, tag) {
            ("Citizens", Tag::List(rows)) => Value::Citizens(
                rows.iter()
                    .filter_map(|row| {
                        let Tag::Compound(row) = row else {
                            return None;
                        };
                        Some((
                            text_of(row, "Id")?,
                            text_of(row, "Name")?,
                            text_of(row, "Job")?,
                        ))
                    })
                    .collect(),
            ),
            ("Requests", Tag::List(rows)) => Value::Requests(
                rows.iter()
                    .filter_map(|row| {
                        let Tag::Compound(row) = row else {
                            return None;
                        };
                        Some(Request {
                            citizen: text_of(row, "Citizen")?,
                            name: text_of(row, "Name")?,
                            job: Job::from_id(&text_of(row, "Job")?)?,
                            item: text_of(row, "Item")?,
                        })
                    })
                    .collect(),
            ),
            (_, Tag::Int(value)) => Value::Int(*value),
            (_, Tag::Long(value)) => Value::Long(*value),
            (_, Tag::String(value)) => Value::Str(value.to_string_lossy()),
            _ => continue,
        };
        fields.push((key, value));
    }
    fields
}

fn text_of(map: &Compound, key: &str) -> Option<String> {
    match map.get(&key.into())? {
        Tag::String(value) => Some(value.to_string_lossy()),
        _ => None,
    }
}

/// Every town hall with a colony, read from the dimension's colony attachments.
fn halls(host: &dyn Host) -> Vec<TownHall> {
    let mut found = Vec::new();
    for (pos, colony) in colonies(host) {
        found.push(TownHall {
            id: colony.id,
            name: colony.name.clone(),
            owner: colony.owner.clone(),
            center: Pos::new(pos[0], pos[1], pos[2]),
        });
    }
    found
}

/// Every `(hall position, colony)` in the dimension, in holder order.
fn colonies(host: &dyn Host) -> Vec<([i32; 3], Colony)> {
    let Ok(query) = host.query(&COLONY) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for row in query.iter() {
        let Ok(row) = row else {
            continue;
        };
        let AttachmentHolder::BlockEntity(entity) = row.holder else {
            continue;
        };
        let Some(pos) = entity.position(host) else {
            continue;
        };
        found.push((pos, row.value.clone()));
    }
    found
}

/// The town hall whose colony is `id`.
fn hall_by_id(host: &dyn Host, id: u32) -> Option<([i32; 3], Colony)> {
    colonies(host)
        .into_iter()
        .find(|(_, colony)| colony.id == id)
}

/// Writes the colony into the town hall at `pos`, replacing what was there.
fn write_colony(host: &mut dyn ServerHost, pos: [i32; 3], colony: &Colony) {
    let _ = COLONY.set(host.attachments(), Holder::BlockEntity(pos), colony.clone());
}

/// Whether the building `building` stands inside the colony: the block a job's rule asks for.
/// One query answers every job — the marker says *which* hut it is (`Job::building`), so a new
/// job needs a constant, not a new attachment.
fn has_building(host: &dyn Host, colony: &Colony, building: &str) -> bool {
    let Ok(query) = host.query(&HUT) else {
        return false;
    };
    query.iter().flatten().any(|row| {
        if row.value.building != building {
            return false;
        }
        let AttachmentHolder::BlockEntity(entity) = row.holder else {
            return false;
        };
        let Some(pos) = entity.position(host) else {
            return false;
        };
        colony.contains(Pos::new(pos[0], pos[1], pos[2]))
    })
}

/// The acting player's colony: the one they own, else the closest town hall that reaches them.
fn player_colony(host: &mut dyn ServerHost, player: PlayerRef) -> Option<([i32; 3], Colony)> {
    let mine = uuid_text(player.uuid());
    let at = player.entity().position(host)?;
    let at = Pos::new(
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    );
    let mut owned = None;
    let mut nearest: Option<(f64, [i32; 3], Colony)> = None;
    for (hall, colony) in colonies(host) {
        if colony.owner == mine {
            owned = Some((hall, colony.clone()));
        }
        let reach = f64::from(radius_blocks(colony.size_chunks) + 32);
        let distance = horizontal_distance(at, Pos::new(hall[0], hall[1], hall[2]));
        if distance <= reach && nearest.as_ref().is_none_or(|(near, _, _)| distance < *near) {
            nearest = Some((distance, hall, colony));
        }
    }
    owned.or_else(|| nearest.map(|(_, hall, colony)| (hall, colony)))
}

/// The colony `player` **owns**, the only one they may manage. Reading a nearby
/// colony (`player_colony`) is a neighbour's courtesy; renaming it, resizing it,
/// hiring into it or reassigning its citizens is the owner's alone.
fn owned_colony(host: &mut dyn ServerHost, player: PlayerRef) -> Option<([i32; 3], Colony)> {
    let mine = uuid_text(player.uuid());
    colonies(host)
        .into_iter()
        .find(|(_, colony)| may_manage(colony, &mine))
}

/// The refusal a management usage answers when the player owns no colony.
const OWNER_ONLY: &str = "Only a colony's owner may manage it: found one with a town hall first.";

// ---------------------------------------------------------------------------
// Declaration: the block entity types and the citizen's Part
// ---------------------------------------------------------------------------

/// The town hall keeps the colony and each hut marks where a job's building stands: all of them
/// get a declared block entity type, which is what creates their block entity (the retired
/// bundle's `block_entity` hook). The colony itself is the `COLONY` attachment on the hall's.
pub(crate) fn register(r: &mut dyn mod_api::Registrar) {
    let hall = r
        .block_entity_type("town_hall")
        .blocks(&["blockhuttownhall"])
        .build();
    // One type per hut; the block id is `HUTS`' own, so the declaration cannot drift from the
    // rule that reads the marker.
    for (type_id, hut) in [
        ("farmer_hut", FARMER_HUT),
        ("lumberjack_hut", LUMBERJACK_HUT),
        ("miner_hut", MINER_HUT),
    ] {
        let block = hut.strip_prefix("minecolonies:").unwrap_or(hut);
        let _ = r.block_entity_type(type_id).blocks(&[block]).build();
    }
    // Entity Parts are native-only: the API attaches code to an entity type, and a wasm mod
    // cannot (yet). A wasm build therefore registers the huts and has no citizen tick.
    #[cfg(not(target_arch = "wasm32"))]
    {
        r.block_entity_part(&hall, crate::parts::TownHallWorker)
            .expect("minecolonies: the town hall takes its Part");
        r.entity_part(CITIZEN, crate::parts::CitizenWorker);
    }
}

// ---------------------------------------------------------------------------
// Founding
// ---------------------------------------------------------------------------

/// What a placement this mod completes once the block exists turns into.
enum PendingKind {
    /// A town hall that founds a colony for the player who placed it.
    Founding {
        player: u128,
        owner: Owner,
        name: String,
    },
    /// A hut that marks where its job's building stands.
    Building { block: String },
}

/// A placement this mod completes once the block exists.
struct Pending {
    dimension: String,
    pos: [i32; 3],
    /// What the placement becomes.
    kind: PendingKind,
    /// Level ticks that did not match this placement's dimension, so a level that never ticks
    /// cannot leave it queued forever.
    misses: u32,
}

/// How many level ticks a pending placement may miss its own dimension before it is dropped.
const PENDING_MISSES: u32 = 200;

/// Placements seen this tick. The block is written after `entity_place` returns, so the colony can
/// only be stored from the next tick.
static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

fn queue(pending: Pending) {
    PENDING
        .lock()
        .expect("the pending-placement queue is never poisoned by a panic-free push")
        .push(pending);
}

/// A block is about to be placed (`BlockEvent.EntityPlaceEvent`): a town hall that could not
/// found a colony is refused, so no town hall stands without its colony. This is the only moment
/// that can stop the write (`block_entity`/`block_entity_tick` only hear about it).
pub(crate) fn on_entity_place(
    host: &mut dyn ServerHost,
    event: &mut mod_api::EntityPlaceEvent,
) -> Flow {
    let placed = event.state.block().id(host);
    if is_hut(&placed) {
        queue(Pending {
            dimension: host.dimension().to_owned(),
            pos: [event.pos.x, event.pos.y, event.pos.z],
            kind: PendingKind::Building { block: placed },
            misses: 0,
        });
        return Flow::Continue;
    }
    if placed != TOWN_HALL {
        return Flow::Continue;
    }
    let pos = [event.pos.x, event.pos.y, event.pos.z];
    // The placer is the player for `BlockItem.place`; the acting player is the fallback for a
    // placement the game makes on their behalf.
    let placer = event
        .placer
        .and_then(|entity| entity.uuid(host))
        .or_else(|| host.players().acting_player().map(PlayerRef::uuid));
    let Some(uuid) = placer else {
        return Flow::Continue;
    };
    let Some(player) = host.player(uuid) else {
        return Flow::Continue;
    };
    let Some(name) = player.name(host) else {
        return Flow::Continue;
    };
    let owner = Owner {
        id: uuid_text(uuid),
        name: name.clone(),
    };
    let colony_name = default_name(&owner.name);
    if let Err(refusal) = founding_refusal(
        &colony_name,
        Some(&owner),
        Pos::new(pos[0], pos[1], pos[2]),
        &halls(host),
    ) {
        // Outside a command `feedback` only reaches the server's own chat, so the refusal is told
        // to the player whose placement it was.
        let _ = player.tell(host, Text::literal(format!("No colony here: {refusal}.")));
        return Flow::Cancel;
    }
    queue(Pending {
        dimension: host.dimension().to_owned(),
        pos,
        kind: PendingKind::Founding {
            player: uuid,
            owner,
            name: colony_name,
        },
        misses: 0,
    });
    Flow::Continue
}

/// Stores a completed placement: the colony in a fresh town hall, the marker in a hut.
fn complete(host: &mut dyn ServerHost, level: LevelRef, item: Pending) {
    let Pending { pos, kind, .. } = item;
    let Some(block) = level.block(host, pos) else {
        return;
    };
    let placed = block.id(host);
    let (uuid, owner, colony_name) = match kind {
        PendingKind::Building { block } => {
            if placed == block {
                let _ = HUT.set(
                    host.attachments(),
                    Holder::BlockEntity(pos),
                    HutMarker { building: block },
                );
            }
            return;
        }
        PendingKind::Founding {
            player,
            owner,
            name,
        } => (player, owner, name),
    };
    if placed != TOWN_HALL {
        return;
    }
    // The use hook already said yes; re-check so a colony can never appear without its rules.
    let Ok(founding) = founding_refusal(
        &colony_name,
        Some(&owner),
        Pos::new(pos[0], pos[1], pos[2]),
        &halls(host),
    ) else {
        return;
    };
    let now = level.game_time(host).unwrap_or(0);
    let colony = Colony::founded(founding, now);
    write_colony(host, pos, &colony);
    let line = format!(
        "Colony {} founded: {} blocks around the town hall, owner {}.",
        colony.name,
        colony.radius(),
        colony.owner_name
    );
    if let Some(player) = host.player(uuid) {
        let _ = player.tell(host, Text::literal(line));
    }
}

/// A player broke a block (`Hooks::player_destroyed_block`): a hut's marker goes with it, so the
/// colony never claims a building that is not standing. `job info` then reads "missing" and the
/// job's tick stops, which is what a break has to mean.
pub(crate) fn on_player_destroyed_block(
    host: &mut dyn ServerHost,
    event: &mod_api::PlayerDestroyedBlockEvent,
) {
    let broken = event.state.block().id(host);
    if !is_hut(&broken) {
        return;
    }
    let _ = HUT.remove(host.attachments(), Holder::BlockEntity(event.pos));
}

/// A level's tick ended on the server: completes the placements queued in that level, then
/// applies the deliveries its workers made. A level-scoped hook, because the colony is written
/// through *this* level's attachment store (`server_tick` does not say which level the host
/// holds) and a block-entity write made *inside* a tick can be overwritten by that entity's own
/// write-back (HANDOVER §9d) — so the chest is filled here, as a delta, not from the job's tick.
pub(crate) fn on_level_tick_post(host: &mut dyn ServerHost, level: LevelRef) {
    // `level.dimension().id(host)` is the level's own dimension name; `server_tick` has no level.
    let dimension = level.dimension().id(host);
    let mut queue = PENDING
        .lock()
        .expect("the pending-placement queue is never poisoned by a panic-free push");
    let mut due = Vec::new();
    let mut waiting = Vec::new();
    for mut item in std::mem::take(&mut *queue) {
        if dimension.as_deref() == Some(item.dimension.as_str()) {
            due.push(item);
        } else {
            item.misses += 1;
            if item.misses < PENDING_MISSES {
                waiting.push(item);
            }
        }
    }
    *queue = waiting;
    drop(queue);
    for item in due {
        complete(host, level, item);
    }
    flush_pickups(host, level, dimension.as_deref());
    flush_deliveries(host, level, dimension.as_deref());
}

// ---------------------------------------------------------------------------
// Deliveries: what a worker collected, applied at the end of that level's tick
// ---------------------------------------------------------------------------

/// One worker's delivery: the drops it collected and, for a felled tree, the sapling its stump
/// gets back. Queued from the job tick and applied at that level's `level_tick_post`.
struct Delivery {
    dimension: String,
    /// The colony chest the drops go into, and the sapling comes out of, when one stands near.
    chest: Option<[i32; 3]>,
    /// Where anything the chest cannot take falls, so a full chest loses nothing.
    at: [i32; 3],
    /// What the worker collected (`destroy_block`'s drops, not item entities).
    items: Vec<ItemStack>,
    /// The replant: the stump's position and the sapling item to take from the chest first.
    replant: Option<([i32; 3], String)>,
    /// Level ticks that did not match this delivery's dimension.
    misses: u32,
}

/// Deliveries seen this tick, applied at the level's end.
static DELIVERIES: Mutex<Vec<Delivery>> = Mutex::new(Vec::new());

fn deliver(delivery: Delivery) {
    DELIVERIES
        .lock()
        .expect("the delivery queue is never poisoned by a panic-free push")
        .push(delivery);
}

/// Applies this level's deliveries: the sapling first (so a full chest can still replant), then
/// the drops, with anything that does not fit falling at the worker's feet.
fn flush_deliveries(host: &mut dyn ServerHost, level: LevelRef, dimension: Option<&str>) {
    let mut queue = DELIVERIES
        .lock()
        .expect("the delivery queue is never poisoned by a panic-free push");
    let mut due = Vec::new();
    let mut waiting = Vec::new();
    for mut item in std::mem::take(&mut *queue) {
        if dimension == Some(item.dimension.as_str()) {
            due.push(item);
        } else {
            item.misses += 1;
            if item.misses < PENDING_MISSES {
                waiting.push(item);
            }
        }
    }
    *queue = waiting;
    drop(queue);
    for item in due {
        if let Some((stump, sapling)) = item.replant
            && take_one(host, level, item.chest, &sapling)
        {
            plant(host, level, stump, &sapling);
        }
        let leftover = insert_drops(host, level, item.chest, &item.items);
        for stack in leftover {
            let _ = level.pop_resource(host, item.at, stack);
        }
    }
}

/// The nearest `minecraft:chest` inside the colony, within a worker's reach of its building:
/// the colony's storage, which is where a worker's harvest goes and its saplings come from.
/// (Upstream delivers to the worker's own building's rack; this slice has no building blocks
/// beyond the farmer's hut, so one chest in the colony is the store — recorded in `notes.md`.)
fn colony_chest(
    host: &dyn Host,
    level: LevelRef,
    at: [f64; 3],
    colony: &Colony,
) -> Option<[i32; 3]> {
    let chest: BlockKey = host.block_key(CHEST)?;
    let centre = Pos::new(
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    );
    let mut best: Option<(f64, [i32; 3])> = None;
    for dx in -STORAGE_SEARCH..=STORAGE_SEARCH {
        for dz in -STORAGE_SEARCH..=STORAGE_SEARCH {
            for dy in -STORAGE_HEIGHT..=STORAGE_HEIGHT {
                let pos = Pos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                if !colony.contains(pos) {
                    continue;
                }
                let block = [pos.x, pos.y, pos.z];
                if level.block(host, block) != Some(chest) {
                    continue;
                }
                let distance = horizontal_distance(centre, pos);
                if best.is_none_or(|(near, _)| distance < near) {
                    best = Some((distance, block));
                }
            }
        }
    }
    best.map(|(_, pos)| pos)
}

/// Puts `items` into the chest, answering what did not fit. No chest (or no storage) keeps
/// everything, which the caller drops.
fn insert_drops(
    host: &mut dyn ServerHost,
    level: LevelRef,
    chest: Option<[i32; 3]>,
    items: &[ItemStack],
) -> Vec<ItemStack> {
    let Some(chest) = chest else {
        return items.to_vec();
    };
    let mut left = items.to_vec();
    let won = host
        .transaction(|host, tx| {
            let Some(storage) = level.capability(host, &mod_api::STORAGE, chest, None) else {
                return Ok::<bool, ()>(false);
            };
            let mut rest = Vec::new();
            for stack in left.drain(..) {
                let resource = ItemResource::of(&stack);
                let count = u64::from(stack.count());
                let put = u32::try_from(storage.insert(host, tx, &resource, count)).unwrap_or(0);
                if let Some(remainder) = stack.shrink(put) {
                    rest.push(remainder);
                }
            }
            left = rest;
            Ok(true)
        })
        .unwrap_or(false);
    if !won {
        return items.to_vec();
    }
    left
}

/// Takes one `item` out of the chest: whether the colony's storage had one to replant with.
fn take_one(
    host: &mut dyn ServerHost,
    level: LevelRef,
    chest: Option<[i32; 3]>,
    item: &str,
) -> bool {
    let Some(chest) = chest else {
        return false;
    };
    let Some(key) = item_key(host, item) else {
        return false;
    };
    let resource = ItemResource::of(&ItemStack::of(key));
    host.transaction(|host, tx| {
        let Some(storage) = level.capability(host, &mod_api::STORAGE, chest, None) else {
            return Ok::<bool, ()>(false);
        };
        Ok(storage.extract(host, tx, &resource, 1) >= 1)
    })
    .unwrap_or(false)
}

/// Plants a sapling on the stump's soil, when the soil grows one and the stump's cell is free.
fn plant(host: &mut dyn ServerHost, level: LevelRef, stump: [i32; 3], sapling: &str) {
    let soil = level
        .block(host, [stump[0], stump[1] - 1, stump[2]])
        .map(|block| block.id(host));
    if !soil.as_deref().is_some_and(grows_a_sapling) {
        return;
    }
    if level
        .block(host, stump)
        .map(|block| block.id(host))
        .as_deref()
        != Some(AIR)
    {
        return;
    }
    let Some(state) = host.block_state_id(sapling) else {
        return;
    };
    let _ = level.set_block(
        host,
        stump,
        state,
        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
    );
}

/// An item key from its id, when the game knows the item.
fn item_key(host: &dyn Host, id: &str) -> Option<mod_api::ItemKey> {
    let identifier = Identifier::parse(id).ok()?;
    host.item_key(&identifier)
}

/// A town hall ticked: step its colony's citizens' jobs. See `parts::TownHallWorker` for why
/// the hall drives them (a vanilla-family entity type hears no `general_entity_tick` at this
/// pin). The colony is read back from the store at the hall's own position, which the tick has.
pub(crate) fn on_hall_tick(host: &mut dyn ServerHost, pos: [i32; 3]) {
    let Some(colony) = COLONY
        .get_existing(host.attachments(), Holder::BlockEntity(pos))
        .ok()
        .flatten()
    else {
        return;
    };
    let Some(level) = host
        .levels()
        .list_levels()
        .into_iter()
        .find(|level| level.in_place(host))
    else {
        return;
    };
    for citizen in &colony.citizens {
        let Some(uuid) = uuid_value(&citizen.id) else {
            continue;
        };
        let Some(entity) = host.entity_by_uuid(mod_api::EntityUuid(uuid)) else {
            continue;
        };
        step_job(host, entity, level, pos, &colony);
    }
}

/// A citizen ticked through its entity Part: resolve its colony by tag and step the job.
/// (The hall's tick uses `on_hall_tick`, which reads its own colony.)
pub(crate) fn on_citizen_tick(host: &mut dyn ServerHost, entity: EntityRef) {
    let Some(level) = entity.level(host) else {
        return;
    };
    let Some(tags) = entity.tags(host) else {
        return;
    };
    let Some(colony_id) = read_colony_tag(&tags) else {
        return;
    };
    let Some((hall, colony)) = hall_by_id(host, colony_id) else {
        return;
    };
    step_job(host, entity, level, hall, &colony);
}

/// One citizen's job step: the job's own delay, its building when it has one, its trade tool,
/// then the job.
fn step_job(
    host: &mut dyn ServerHost,
    entity: EntityRef,
    level: LevelRef,
    hall: [i32; 3],
    colony: &Colony,
) {
    let Some(uuid) = entity.uuid(host) else {
        return;
    };
    let id = uuid_text(uuid);
    let Some(citizen) = colony.citizen(&id) else {
        return;
    };
    let job = citizen.job;
    let name = citizen.name.clone();
    let Some(delay) = job.delay() else {
        return;
    };
    // A job that needs its building does not work without it (upstream's `AbstractJob` has a
    // building per job; every working job in this slice has its own hut block).
    if let Some(building) = job.building()
        && !has_building(host, colony, building)
    {
        return;
    }
    let Some(now) = level.game_time(host) else {
        return;
    };
    if !should_work(now, entity.id(), delay) {
        return;
    }
    // A job's rule needs its trade tool in the citizen's hand. Without it the worker asks the
    // colony for one — the request — and does not work; the tool is taken from the colony's
    // store at the level's end (`Pickup`), and the request is settled when it is in the hand.
    if let Some(tool) = job.tool() {
        let held = held_item(&*host, entity);
        if tool_missing(job, held.as_deref()) {
            let request = Request {
                citizen: id,
                name,
                job,
                item: tool.item.to_owned(),
            };
            update_colony(host, hall, |colony| colony.request(request));
            if let Some(at) = entity.position(host) {
                let chest = colony_chest(host, level, at, colony);
                ask_for_tool(Pickup {
                    dimension: host.dimension().to_owned(),
                    hall,
                    citizen: uuid,
                    item: tool.item.to_owned(),
                    chest,
                    misses: 0,
                });
            }
            return;
        }
        update_colony(host, hall, |colony| colony.settle(&id));
    }
    match job {
        Job::Farmer => farmer_work(host, entity, level, hall, colony),
        Job::Lumberjack => lumberjack_work(host, entity, level, hall, colony),
        Job::Miner => miner_work(host, entity, level, hall, colony),
        Job::Unemployed => {}
    }
}

/// The item id in an entity's main hand, or `None` for an empty hand (or an unreadable entity).
fn held_item(host: &dyn Host, entity: EntityRef) -> Option<String> {
    let living = host.living(entity)?;
    let stack = living.hand_item(host, Hand::Main)?;
    Some(stack.item().id(host))
}

/// The item id in a roster citizen's main hand, resolved through its entity uuid.
fn citizen_hand(host: &mut dyn ServerHost, citizen: &str) -> Option<String> {
    let uuid = uuid_value(citizen)?;
    let entity = host.entity_by_uuid(mod_api::EntityUuid(uuid))?;
    held_item(host, entity)
}

// ---------------------------------------------------------------------------
// Tool pickups: a requested tool is taken from the colony's store at the level's end
// ---------------------------------------------------------------------------

/// A tool a worker asked its colony for: taken from the colony's store, and put in the citizen's
/// main hand, at that level's `level_tick_post`. A block-entity write made inside a tick is
/// overwritten by that entity's own write-back (the same rule the deliveries follow), so the
/// chest is touched here and not in the job tick.
struct Pickup {
    dimension: String,
    /// The colony's town hall, so the request the pickup settles is written to the right record.
    hall: [i32; 3],
    /// The citizen, as its entity uuid.
    citizen: u128,
    /// The item the job needs.
    item: String,
    /// The colony's store, when one stands inside the colony near the worker.
    chest: Option<[i32; 3]>,
    /// Level ticks that did not match this pickup's dimension.
    misses: u32,
}

static PICKUPS: Mutex<Vec<Pickup>> = Mutex::new(Vec::new());

fn ask_for_tool(pickup: Pickup) {
    PICKUPS
        .lock()
        .expect("the pickup queue is never poisoned by a panic-free push")
        .push(pickup);
}

/// Gives each waiting worker the tool its colony holds, and settles the request it made.
fn flush_pickups(host: &mut dyn ServerHost, level: LevelRef, dimension: Option<&str>) {
    let mut queue = PICKUPS
        .lock()
        .expect("the pickup queue is never poisoned by a panic-free push");
    let mut due = Vec::new();
    let mut waiting = Vec::new();
    for mut item in std::mem::take(&mut *queue) {
        if dimension == Some(item.dimension.as_str()) {
            due.push(item);
        } else {
            item.misses += 1;
            if item.misses < PENDING_MISSES {
                waiting.push(item);
            }
        }
    }
    *queue = waiting;
    drop(queue);
    for item in due {
        let Some(entity) = host.entity_by_uuid(mod_api::EntityUuid(item.citizen)) else {
            continue;
        };
        let id = uuid_text(item.citizen);
        if held_item(&*host, entity).as_deref() == Some(item.item.as_str()) {
            update_colony(host, item.hall, |colony| colony.settle(&id));
            continue;
        }
        let Some(living) = host.living(entity) else {
            continue;
        };
        let Some(key) = item_key(host, &item.item) else {
            continue;
        };
        if !take_one(host, level, item.chest, &item.item) {
            // The colony has no such tool (yet): the request stays open.
            continue;
        }
        let stack = ItemStack::of(key);
        if living
            .set_hand_item(host, Hand::Main, Some(stack.clone()))
            .unwrap_or(false)
        {
            update_colony(host, item.hall, |colony| colony.settle(&id));
            continue;
        }
        // The hand could not be written: the tool goes back to the store, and to the ground if
        // the store cannot take it, so a failed write never loses the item.
        let leftover = insert_drops(host, level, item.chest, std::slice::from_ref(&stack));
        if let Some(at) = entity.position(host) {
            let feet = [
                at[0].floor() as i32,
                at[1].floor() as i32,
                at[2].floor() as i32,
            ];
            for stack in leftover {
                let _ = level.pop_resource(host, feet, stack);
            }
        }
    }
}

/// Applies `change` to the colony as it stands *now* at `hall`, then writes it back when `change`
/// says it altered the record. A worker's counter is a delta on the live record, so two citizens
/// that work on the same tick cannot lose each other's increment (HANDOVER §9d: a write is
/// applied as a delta, not from a stale copy).
fn update_colony(
    host: &mut dyn ServerHost,
    hall: [i32; 3],
    change: impl FnOnce(&mut Colony) -> bool,
) {
    let Some(mut colony) = COLONY
        .get_existing(host.attachments(), Holder::BlockEntity(hall))
        .ok()
        .flatten()
    else {
        return;
    };
    if change(&mut colony) {
        write_colony(host, hall, &colony);
    }
}

/// A counter bump: always a change, so the record is always written back.
fn bump_colony(host: &mut dyn ServerHost, hall: [i32; 3], change: impl FnOnce(&mut Colony)) {
    update_colony(host, hall, |colony| {
        change(colony);
        true
    });
}

/// The farmer's work tick: walk to the nearest mature crop, harvest it and leave it replanted.
fn farmer_work(
    host: &mut dyn ServerHost,
    entity: EntityRef,
    level: LevelRef,
    hall: [i32; 3],
    colony: &Colony,
) {
    let Some(at) = entity.position(host) else {
        return;
    };
    let Some(crop) = nearest_mature_crop(host, level, at, colony) else {
        return;
    };
    match farmer_step(at, Some(crop), FARMER_REACH) {
        Some(Step::Walk(to)) => {
            // A `NoAI` citizen is moved by its job: one stride, the height it stands at.
            entity.teleport_to(host, to, None);
        }
        Some(Step::Harvest(at_crop)) => {
            let pos = [at_crop.x, at_crop.y, at_crop.z];
            if host.set_block_property(pos, "age", WHEAT_REPLANT_AGE) {
                bump_colony(host, hall, Colony::harvest);
            }
        }
        None | Some(Step::Fell(_)) => {}
    }
}

/// The lumberjack's work tick: find the tree nearest to it inside the colony, walk to it and
/// fell one log — the highest one first. The last log puts a sapling back on the stump's soil
/// when the colony's chest holds one (upstream's lumberjack replants from its own inventory).
fn lumberjack_work(
    host: &mut dyn ServerHost,
    entity: EntityRef,
    level: LevelRef,
    hall: [i32; 3],
    colony: &Colony,
) {
    let Some(at) = entity.position(host) else {
        return;
    };
    let Some(tree) = nearest_tree(host, level, at, colony) else {
        return;
    };
    if tree.is_empty() {
        return;
    }
    match lumberjack_step(at, Some(&tree), LUMBERJACK_REACH) {
        Some(Step::Walk(to)) => {
            entity.teleport_to(host, to, None);
        }
        Some(Step::Fell(log)) => {
            let pos = [log.x, log.y, log.z];
            let Ok(drops) = level.destroy_block(host, pos, true, None, true) else {
                return;
            };
            let Some(drops) = drops else {
                return;
            };
            bump_colony(host, hall, |colony| colony.count_felled(1));
            // The last log is the stump: its soil gets the tree's sapling back, if there is one.
            let replant = if tree.len() == 1 {
                tree.stump()
                    .zip(sapling_for(&tree.log))
                    .map(|(stump, sapling)| ([stump.x, stump.y, stump.z], sapling))
            } else {
                None
            };
            let chest = colony_chest(host, level, at, colony);
            deliver(Delivery {
                dimension: host.dimension().to_owned(),
                chest,
                at: [
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32,
                ],
                items: drops,
                replant,
                misses: 0,
            });
        }
        _ => {}
    }
}

/// The miner's work tick: break the first block under the shaft it stands over, one per tick,
/// and deliver the drops. It stops at bedrock, at water or lava, at the world floor and outside
/// the colony (`runtime::miner_next`).
fn miner_work(
    host: &mut dyn ServerHost,
    entity: EntityRef,
    level: LevelRef,
    hall: [i32; 3],
    colony: &Colony,
) {
    let Some(at) = entity.position(host) else {
        return;
    };
    let stand = Pos::new(
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    );
    let floor = level.height(host).map_or(i32::MIN, |(min, _)| min);
    match miner_next(stand, colony, floor, |pos| {
        level
            .block(host, [pos.x, pos.y, pos.z])
            .map(|block| block.id(host))
    }) {
        crate::runtime::Mining::Next(block) => {
            let pos = [block.x, block.y, block.z];
            let Ok(drops) = level.destroy_block(host, pos, true, None, true) else {
                return;
            };
            let Some(drops) = drops else {
                return;
            };
            bump_colony(host, hall, |colony| colony.count_dug(1));
            let chest = colony_chest(host, level, at, colony);
            deliver(Delivery {
                dimension: host.dimension().to_owned(),
                chest,
                at: [
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32,
                ],
                items: drops,
                replant: None,
                misses: 0,
            });
        }
        crate::runtime::Mining::Stop(_) => {}
    }
}

/// The tree nearest to `at` inside the colony: the log, its whole connected cluster.
fn nearest_tree(
    host: &dyn Host,
    level: LevelRef,
    at: [f64; 3],
    colony: &Colony,
) -> Option<crate::runtime::Tree> {
    let centre = Pos::new(
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    );
    let mut best: Option<(f64, Pos, String)> = None;
    for dx in -LUMBERJACK_SEARCH..=LUMBERJACK_SEARCH {
        for dz in -LUMBERJACK_SEARCH..=LUMBERJACK_SEARCH {
            for dy in -LOG_SCAN_HEIGHT..=LOG_SCAN_HEIGHT {
                let pos = Pos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                if !colony.contains(pos) {
                    continue;
                }
                let Some(id) = level
                    .block(host, [pos.x, pos.y, pos.z])
                    .map(|block| block.id(host))
                else {
                    continue;
                };
                if !is_log(&id) {
                    continue;
                }
                let distance = horizontal_distance(centre, pos);
                if best.as_ref().is_none_or(|(near, _, _)| distance < *near) {
                    best = Some((distance, pos, id));
                }
            }
        }
    }
    let (_, seed, log) = best?;
    Some(tree_at(seed, &log, |pos| {
        level
            .block(host, [pos.x, pos.y, pos.z])
            .map(|block| block.id(host))
    }))
}

/// The colony's nearest mature wheat around `at`, all of it inside the colony.
fn nearest_mature_crop(
    host: &dyn Host,
    level: LevelRef,
    at: [f64; 3],
    colony: &Colony,
) -> Option<Pos> {
    let centre = Pos::new(
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    );
    let wheat: Option<BlockKey> = host.block_key(WHEAT);
    let mut best: Option<(f64, Pos)> = None;
    for dx in -CROP_RANGE..=CROP_RANGE {
        for dz in -CROP_RANGE..=CROP_RANGE {
            for dy in -CROP_HEIGHT..=CROP_HEIGHT {
                let pos = Pos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                if !colony.contains(pos) {
                    continue;
                }
                let block = [pos.x, pos.y, pos.z];
                if level.block(host, block) != wheat {
                    continue;
                }
                if level.block_property(host, block, "age").as_deref() != Some(WHEAT_MAX_AGE) {
                    continue;
                }
                let distance = horizontal_distance(centre, pos);
                if best.as_ref().is_none_or(|(near, _)| distance < *near) {
                    best = Some((distance, pos));
                }
            }
        }
    }
    best.map(|(_, pos)| pos)
}

// ---------------------------------------------------------------------------
// `/minecolonies colony|citizen|job`
// ---------------------------------------------------------------------------

/// The commands (`register_commands`). `citizens` is the mod's own spelling (`EntryPoint.java:70`),
/// kept beside the slice's `citizen`; `mc` is the mod's alias.
pub(crate) fn on_register_commands(event: &mut RegisterCommandsEvent<'_>) {
    for root in ["minecolonies", "mc"] {
        event
            .commands
            .push(Command::new(&format!("{root} colony info")));
        event
            .commands
            .push(Command::new(&format!("{root} colony rename <name:text>")));
        event.commands.push(Command::new(&format!(
            "{root} colony radius <chunks:integer>"
        )));
        event
            .commands
            .push(Command::new(&format!("{root} requests")));
        for citizen in ["citizen", "citizens"] {
            event
                .commands
                .push(Command::new(&format!("{root} {citizen} hire [name:text]")));
            event
                .commands
                .push(Command::new(&format!("{root} {citizen} list")));
            event.commands.push(Command::new(&format!(
                "{root} job assign <citizen:word> <job:word>"
            )));
            event
                .commands
                .push(Command::new(&format!("{root} job info <citizen:word>")));
        }
    }
}

fn usage() -> String {
    "usage: /minecolonies colony info|rename <name>|radius <chunks> · citizen hire [name]|list · job assign <citizen> <job>|info <citizen> · requests"
        .to_owned()
}

/// Runs a parsed `/minecolonies …` usage.
pub(crate) fn on_run_mod_command(
    host: &mut dyn ServerHost,
    event: &mut RunModCommandEvent<'_>,
) -> Flow {
    let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
    let Some(root) = literals.first().copied() else {
        return Flow::Continue;
    };
    if root != "minecolonies" && root != "mc" {
        return Flow::Continue;
    }
    let player = match event.source {
        CommandSource::Player { player, .. } => Some(*player),
        _ => host.players().acting_player(),
    };
    let Some(player) = player else {
        feedback(host, "This command needs a player.".to_owned());
        return Flow::Cancel;
    };
    match literals.get(1).copied() {
        Some("colony") => match literals.get(2).copied() {
            Some("rename") => rename(host, player, event),
            Some("radius") => radius(host, player, event),
            _ => info(host, player),
        },
        Some("citizen" | "citizens") => match literals.get(2).copied() {
            Some("hire") => hire(host, player, event),
            _ => list(host, player),
        },
        Some("job") => match literals.get(2).copied() {
            Some("assign") => assign(host, player, event),
            Some("info") => info_of(host, player, event),
            _ => feedback(host, usage()),
        },
        Some("requests") => open_requests(host, player),
        _ => feedback(host, usage()),
    }
    Flow::Cancel
}

/// The `text` argument of a usage, trimmed; an absent one is `None`.
fn text_argument(event: &RunModCommandEvent<'_>, name: &str) -> Option<String> {
    match event.command.argument(name)? {
        CommandArgumentValue::String(value) => {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        }
        _ => None,
    }
}

/// The `word` argument of a usage.
fn word_argument(event: &RunModCommandEvent<'_>, name: &str) -> Option<String> {
    match event.command.argument(name)? {
        CommandArgumentValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn feedback(host: &mut dyn ServerHost, line: String) {
    let _ = host.commands().feedback(Text::literal(line));
}

fn no_colony(host: &mut dyn ServerHost) {
    feedback(
        host,
        "You have no colony: place a town hall to found one.".to_owned(),
    );
}

fn info(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some((_, colony)) = player_colony(host, player) else {
        no_colony(host);
        return;
    };
    feedback(host, colony_summary(&colony));
}

fn rename(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = owned_colony(host, player) else {
        feedback(host, OWNER_ONLY.to_owned());
        return;
    };
    let Some(name) = text_argument(event, "name") else {
        feedback(host, usage());
        return;
    };
    match crate::runtime::validate_name(&name) {
        Ok(name) => {
            colony.name = name;
            write_colony(host, hall, &colony);
            feedback(host, colony_summary(&colony));
        }
        Err(refusal) => feedback(host, format!("{refusal}.")),
    }
}

fn radius(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = owned_colony(host, player) else {
        feedback(host, OWNER_ONLY.to_owned());
        return;
    };
    let Some(CommandArgumentValue::Integer(chunks)) = event.command.argument("chunks") else {
        feedback(
            host,
            "usage: /minecolonies colony radius <chunks>".to_owned(),
        );
        return;
    };
    colony.size_chunks = clamp_size(*chunks);
    write_colony(host, hall, &colony);
    feedback(
        host,
        format!(
            "Colony {} now claims {} blocks ({} chunks, max {}).",
            colony.name,
            colony.radius(),
            colony.size_chunks,
            MAX_COLONY_SIZE_CHUNKS
        ),
    );
}

fn list(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some((_, colony)) = player_colony(host, player) else {
        no_colony(host);
        return;
    };
    let mut line = format!(
        "Colony {}: {} citizens.",
        colony.name,
        colony.citizens.len()
    );
    for (index, citizen) in colony.citizens.iter().enumerate() {
        let need = colony
            .open_request(&citizen.id)
            .map(|request| format!(", needs {}", request.item))
            .unwrap_or_default();
        line.push_str(&format!(
            "\n{}. {} — {}{need}",
            index + 1,
            citizen.name,
            citizen.job.label()
        ));
    }
    feedback(host, line);
}

fn hire(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = owned_colony(host, player) else {
        feedback(host, OWNER_ONLY.to_owned());
        return;
    };
    // A hire spawns a persistent mob and saves a roster row: the cap is what bounds
    // both, and it is checked before a mob exists.
    if let Some(refusal) = hire_refusal(&colony) {
        feedback(host, format!("{refusal}."));
        return;
    }
    let name = text_argument(event, "name")
        .unwrap_or_else(|| NAMES[colony.citizens.len() % NAMES.len()].to_owned());
    let spawn = spawn_point(hall, colony.citizens.len());
    // The command's own execution dimension (`execute in` applies to it), not the level that
    // happens to be in place.
    let dimension = event
        .command
        .dimension
        .id(host)
        .unwrap_or_else(|| host.dimension().to_owned());
    let Some(level) = host.level(&dimension) else {
        feedback(host, "The citizen could not be registered.".to_owned());
        return;
    };
    // The content type when the game knows it; the villager it copies otherwise, so a missing
    // content half is visible in the harness (the entity's kind is asserted) rather than silent.
    let mut spawned = None;
    for kind in [CITIZEN, VANILLA_VILLAGER] {
        spawned = level.spawn_mob(host, kind, spawn, SUMMONED);
        if spawned.is_some() {
            break;
        }
    }
    let Some(entity) = spawned else {
        feedback(
            host,
            format!("No room for a citizen at {spawn:?}: load the chunks around the town hall."),
        );
        return;
    };
    let _ = entity.set_custom_name(host, Some(Text::literal(name.clone())));
    // `NoAI`: a citizen is stepped by its job, not by the mob's own AI (the retired bundle's
    // `NativeSpawn { no_ai: true }`).
    MobRef::new(entity).set_no_ai(host, true);
    let Some(uuid) = entity.uuid(host) else {
        feedback(host, "The citizen could not be registered.".to_owned());
        return;
    };
    entity.add_tag(host, &colony_tag(colony.id));
    colony.hire(Citizen {
        id: uuid_text(uuid),
        name: name.clone(),
        job: Job::Unemployed,
    });
    write_colony(host, hall, &colony);
    feedback(
        host,
        format!(
            "{name} joined {} ({} citizens). Give them a job: /minecolonies job assign {name} farmer|lumberjack|miner.",
            colony.name,
            colony.citizens.len()
        ),
    );
}

fn info_of(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((_, colony)) = player_colony(host, player) else {
        no_colony(host);
        return;
    };
    let Some(who) = word_argument(event, "citizen") else {
        feedback(host, usage());
        return;
    };
    let Some(index) = citizen_index(&colony, &who) else {
        feedback(host, format!("{} has no citizen {who}.", colony.name));
        return;
    };
    let citizen = &colony.citizens[index];
    let building = citizen
        .job
        .building()
        .is_some_and(|building| has_building(host, &colony, building));
    let held = citizen_hand(host, &citizen.id);
    feedback(
        host,
        format!(
            "{}: job {}, colony {} ({} crops harvested), {} logs felled, {} blocks mined, {}, {}.",
            citizen.name,
            citizen.job.label(),
            colony.name,
            colony.harvested,
            colony.felled,
            colony.dug,
            crate::runtime::building_status(citizen.job, building),
            hand_status(citizen.job, held.as_deref())
        ),
    );
}

/// `/minecolonies requests`: the colony's open tool requests. The saved list is the record, and
/// it is brought in line with what the citizens hold right now before it is printed, so a row
/// whose tool arrived (or whose citizen lost its job or building) cannot survive a read.
fn open_requests(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some((hall, mut colony)) = player_colony(host, player) else {
        no_colony(host);
        return;
    };
    if reconcile_requests(host, &mut colony) {
        write_colony(host, hall, &colony);
    }
    feedback(host, requests_line(&colony.requests));
}

/// The request rule over the live colony: a citizen whose job has its building but not its tool
/// has an open request; one holding the tool — or with no working job — has none. Answers
/// whether the record changed.
fn reconcile_requests(host: &mut dyn ServerHost, colony: &mut Colony) -> bool {
    let roster: Vec<(String, String, Job)> = colony
        .citizens
        .iter()
        .map(|citizen| (citizen.id.clone(), citizen.name.clone(), citizen.job))
        .collect();
    let mut changed = false;
    for (id, name, job) in roster {
        let working = job
            .building()
            .is_none_or(|building| has_building(host, colony, building));
        match job.tool().filter(|_| working) {
            Some(tool) => {
                if citizen_hand(host, &id).as_deref() == Some(tool.item) {
                    changed |= colony.settle(&id);
                } else {
                    changed |= colony.request(Request {
                        citizen: id,
                        name,
                        job,
                        item: tool.item.to_owned(),
                    });
                }
            }
            None => changed |= colony.settle(&id),
        }
    }
    changed
}

fn assign(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = owned_colony(host, player) else {
        feedback(host, OWNER_ONLY.to_owned());
        return;
    };
    let (Some(who), Some(job_id)) = (word_argument(event, "citizen"), word_argument(event, "job"))
    else {
        feedback(host, usage());
        return;
    };
    let Some(index) = citizen_index(&colony, &who) else {
        feedback(host, format!("{} has no citizen {who}.", colony.name));
        return;
    };
    let Some(job) = Job::from_id(&job_id) else {
        feedback(
            host,
            format!("Unknown job {job_id}: the colony's jobs are farmer, lumberjack and miner."),
        );
        return;
    };
    if let Some(building) = job.building()
        && !has_building(host, &colony, building)
    {
        let label = job.building_label().unwrap_or("its building");
        feedback(
            host,
            format!(
                "A {} needs {label} in the colony: place {building}.",
                job.label().to_lowercase()
            ),
        );
        return;
    }
    let who_id = colony.citizens[index].id.clone();
    if colony.assign(&who_id, job).is_err() {
        return;
    }
    // The request belonged to the job that is gone: the new job's tool is asked for on its own
    // first work tick (or by `/minecolonies requests`), never carried over from the old one.
    colony.settle(&who_id);
    let citizen = &colony.citizens[index];
    feedback(
        host,
        format!(
            "{} is now the colony's {}.",
            citizen.name,
            job.label().to_lowercase()
        ),
    );
    write_colony(host, hall, &colony);
}

// ---------------------------------------------------------------------------
// Citizens
// ---------------------------------------------------------------------------

/// Where a hired citizen appears: beside the town hall, spread along +x by roster size.
fn spawn_point(hall: [i32; 3], index: usize) -> [f64; 3] {
    [
        f64::from(hall[0]) + 2.5 + index as f64,
        f64::from(hall[1]),
        f64::from(hall[2]) + 0.5,
    ]
}

/// The roster index of the citizen named `who` or numbered `who` (1-based).
fn citizen_index(colony: &Colony, who: &str) -> Option<usize> {
    if let Ok(number) = who.parse::<usize>()
        && number >= 1
        && number <= colony.citizens.len()
    {
        return Some(number - 1);
    }
    colony
        .citizens
        .iter()
        .position(|citizen| citizen.name.eq_ignore_ascii_case(who))
}

fn colony_summary(colony: &Colony) -> String {
    let mut line = format!(
        "Colony {} (id {}, owner {}): centre {}, {} blocks ({} chunks), {} citizens, {} crops harvested, {} logs felled, {} blocks mined, {}, founded at tick {}.",
        colony.name,
        colony.id,
        colony.owner_name,
        colony.center,
        colony.radius(),
        colony.size_chunks,
        colony.citizens.len(),
        colony.harvested,
        colony.felled,
        colony.dug,
        requests_status(colony.requests.len()),
        colony.founded
    );
    if colony.citizens.is_empty() {
        line.push_str("\nNo citizens: /minecolonies citizen hire.");
    } else {
        let jobs = colony
            .citizens
            .iter()
            .map(|citizen| format!("{} ({})", citizen.name, citizen.job.label()))
            .collect::<Vec<_>>()
            .join(", ");
        line.push_str(&format!("\nCitizens: {jobs}."));
    }
    line
}

/// A UUID's text form, as `/data` prints one.
fn uuid_value(text: &str) -> Option<u128> {
    let hex: String = text.chars().filter(|character| *character != '-').collect();
    if hex.len() != 32 {
        return None;
    }
    u128::from_str_radix(&hex, 16).ok()
}

fn uuid_text(uuid: u128) -> String {
    let hex = format!("{uuid:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roster_id_round_trips_to_the_entity_uuid() {
        // The roster saves the hyphenated text; `EntityUuid` takes the raw 128-bit value.
        let uuid = 0x0123_4567_89ab_cdef_0011_2233_4455_6677;
        assert_eq!(uuid_value(&uuid_text(uuid)), Some(uuid));
        assert_eq!(uuid_value("not-a-uuid"), None);
        assert_eq!(uuid_value("0123456789abcdef"), None);
    }

    #[test]
    fn a_colony_round_trips_through_its_saved_compound() {
        // The saved form is the only way a colony survives a relaunch: every field the record
        // carries has to come back, including the counters a later slice added and the requests
        // this one does. A key the decoder skips is a field that silently resets on load.
        let colony = Colony {
            id: 4,
            name: "Riverside".to_owned(),
            owner: "00000000-0000-0000-0000-000000000001".to_owned(),
            owner_name: "Builder".to_owned(),
            center: Pos::new(200, 101, 0),
            size_chunks: 4,
            founded: 1234,
            harvested: 9,
            felled: 3,
            dug: 5,
            requests: vec![crate::runtime::Request {
                citizen: "c-2".to_owned(),
                name: "Bo".to_owned(),
                job: Job::Lumberjack,
                item: crate::runtime::LUMBERJACK_TOOL.to_owned(),
            }],
            citizens: vec![Citizen {
                id: "c-2".to_owned(),
                name: "Bo".to_owned(),
                job: Job::Lumberjack,
            }],
        };
        let saved = encode_colony(&colony).expect("a colony encodes");
        assert_eq!(decode_colony(&saved), Ok(colony));
    }

    #[test]
    fn a_hut_marker_saves_which_building_it_is() {
        // The marker is what `job info` reads and what a break removes; the building it names is
        // the whole point, so the saved form has to carry it byte for byte.
        let marker = HutMarker {
            building: LUMBERJACK_HUT.to_owned(),
        };
        let saved = encode_hut(&marker).expect("a marker encodes");
        assert_eq!(text_of(&saved, "Building").as_deref(), Some(LUMBERJACK_HUT));
        assert_eq!(decode_hut(&saved), Ok(marker));
        // A marker written without its building is not a hut this port can answer for: 0.2.4's
        // empty compound must not read as any building.
        assert_eq!(decode_hut(&Compound::new()), Err(AttachmentError::Decode));
    }
}
