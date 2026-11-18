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
    BlockKey, Command, CommandArgumentValue, CommandSource, Compound, EntityRef, Flow, Holder,
    Host, LevelRef, MobRef, PlayerRef, RegisterCommandsEvent, RunModCommandEvent, ServerHost,
    SpawnReason, Tag, Text,
};

use crate::runtime::{
    CITIZEN, Citizen, Colony, FARMER_DELAY_TICKS, FARMER_HUT, FARMER_REACH, Job,
    MAX_COLONY_SIZE_CHUNKS, Owner, Pos, Step, TOWN_HALL, TownHall, Value, WHEAT, WHEAT_MAX_AGE,
    WHEAT_REPLANT_AGE, clamp_size, colony_fields, colony_from_fields, colony_tag, default_name,
    farmer_step, founding_refusal, horizontal_distance, radius_blocks, read_colony_tag,
    should_work,
};

/// The vanilla mob a citizen falls back to when the content type is not loaded.
const VANILLA_VILLAGER: &str = "minecraft:villager";
/// The names a hired citizen can get: our own list, not the mod's name files.
const NAMES: [&str; 8] = ["Ada", "Bo", "Cass", "Dee", "Eli", "Fay", "Gus", "Hana"];
/// The block a citizen's crop search looks at, horizontally.
const CROP_RANGE: i32 = 6;
/// Vertically.
const CROP_HEIGHT: i32 = 2;
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

/// A farmer's hut standing in a colony: the building a farmer's job needs.
static HUT: AttachmentType<HutMarker> = AttachmentType {
    namespace: "minecolonies",
    name: "farmer_hut",
    default: HutMarker::default,
    codec: Some(AttachmentCodec {
        encode: encode_hut,
        decode: decode_hut,
    }),
    copy_on_death: false,
    sync: false,
};

/// The farmer's hut marker's value: the block is the information, so there is nothing to store.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HutMarker;

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
            Value::Text(value) => Tag::String(value.as_str().into()),
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

fn encode_hut(_marker: &HutMarker) -> Result<Compound, AttachmentError> {
    Ok(Compound::new())
}

fn decode_hut(_saved: &Compound) -> Result<HutMarker, AttachmentError> {
    Ok(HutMarker)
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
        "Citizens",
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
            (_, Tag::Int(value)) => Value::Int(*value),
            (_, Tag::Long(value)) => Value::Long(*value),
            (_, Tag::String(value)) => Value::Text(value.to_string_lossy()),
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

/// Whether a farmer's hut stands inside the colony: the building a farmer's job needs.
fn has_farmers_hut(host: &dyn Host, colony: &Colony) -> bool {
    let Ok(query) = host.query(&HUT) else {
        return false;
    };
    query.iter().flatten().any(|row| {
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

// ---------------------------------------------------------------------------
// Declaration: the block entity types and the citizen's Part
// ---------------------------------------------------------------------------

/// The town hall keeps the colony and the farmer's hut marks where a farmer may work: both get a
/// declared block entity type, which is what creates their block entity (the retired bundle's
/// `block_entity` hook). The colony itself is the `COLONY` attachment on that block entity.
pub(crate) fn register(r: &mut dyn mod_api::Registrar) {
    let _ = r
        .block_entity_type("town_hall")
        .blocks(&["blockhuttownhall"])
        .build();
    let _ = r
        .block_entity_type("farmer_hut")
        .blocks(&["blockhutfarmer"])
        .build();
    // Entity Parts are native-only: the API attaches code to an entity type, and a wasm mod
    // cannot (yet). A wasm build therefore registers the huts and has no citizen tick.
    #[cfg(not(target_arch = "wasm32"))]
    r.entity_part(CITIZEN, crate::parts::CitizenWorker);
}

// ---------------------------------------------------------------------------
// Founding
// ---------------------------------------------------------------------------

/// A placement this mod completes once the block exists.
struct Pending {
    dimension: String,
    pos: [i32; 3],
    /// The owner a town hall is founded for; `None` for a hut marker.
    owner: Option<(u128, Owner, String)>,
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
    if placed == FARMER_HUT {
        queue(Pending {
            dimension: host.dimension().to_owned(),
            pos: [event.pos.x, event.pos.y, event.pos.z],
            owner: None,
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
        owner: Some((uuid, owner, colony_name)),
        misses: 0,
    });
    Flow::Continue
}

/// A level's tick ended on the server: completes the placements queued in that level. A
/// level-scoped hook, because the colony is written through *this* level's attachment store
/// (`server_tick` does not say which level the host holds).
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
}

/// Stores a completed placement: the colony in a fresh town hall, the marker in a farmer's hut.
fn complete(host: &mut dyn ServerHost, level: LevelRef, item: Pending) {
    let Some(block) = level.block(host, item.pos) else {
        return;
    };
    let placed = block.id(host);
    let Some((uuid, owner, colony_name)) = item.owner else {
        if placed == FARMER_HUT {
            let _ = HUT.set(host.attachments(), Holder::BlockEntity(item.pos), HutMarker);
        }
        return;
    };
    if placed != TOWN_HALL {
        return;
    }
    // The use hook already said yes; re-check so a colony can never appear without its rules.
    let Ok(founding) = founding_refusal(
        &colony_name,
        Some(&owner),
        Pos::new(item.pos[0], item.pos[1], item.pos[2]),
        &halls(host),
    ) else {
        return;
    };
    let now = level.game_time(host).unwrap_or(0);
    let colony = Colony::founded(founding, now);
    write_colony(host, item.pos, &colony);
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

// ---------------------------------------------------------------------------
// The citizen's job tick
// ---------------------------------------------------------------------------

/// A citizen ticked (the `EntityPart` this mod attaches to `minecolonies:citizen`): its job works
/// if its colony resolves and it has one.
pub(crate) fn on_citizen_tick(host: &mut dyn ServerHost, entity: EntityRef) {
    let Some(level) = entity.level(host) else {
        return;
    };
    let Some(tags) = entity.tags(host) else {
        return;
    };
    // A citizen with no colony tag does nothing at all.
    let Some(colony_id) = read_colony_tag(&tags) else {
        return;
    };
    let Some((hall, mut colony)) = hall_by_id(host, colony_id) else {
        return;
    };
    let Some(uuid) = entity.uuid(host) else {
        return;
    };
    let Some(job) = colony.citizen(&uuid_text(uuid)).map(|citizen| citizen.job) else {
        return;
    };
    if job != Job::Farmer {
        return;
    }
    // The job needs its building: a farmer works only where the colony has a farmer's hut.
    if !has_farmers_hut(host, &colony) {
        return;
    }
    let Some(now) = level.game_time(host) else {
        return;
    };
    if !should_work(now, entity.id(), FARMER_DELAY_TICKS) {
        return;
    }
    let Some(at) = entity.position(host) else {
        return;
    };
    let Some(crop) = nearest_mature_crop(host, level, at, &colony) else {
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
                colony.harvest();
                write_colony(host, hall, &colony);
            }
        }
        None => {}
    }
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
    "usage: /minecolonies colony info|rename <name>|radius <chunks> · citizen hire [name]|list · job assign <citizen> <job>|info <citizen>"
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
    let Some((hall, mut colony)) = player_colony(host, player) else {
        no_colony(host);
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
    let Some((hall, mut colony)) = player_colony(host, player) else {
        no_colony(host);
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
        line.push_str(&format!(
            "\n{}. {} — {}",
            index + 1,
            citizen.name,
            citizen.job.label()
        ));
    }
    feedback(host, line);
}

fn hire(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = player_colony(host, player) else {
        no_colony(host);
        return;
    };
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
            "{name} joined {} ({} citizens). Give them a job: /minecolonies job assign {name} farmer.",
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
    let hut = has_farmers_hut(host, &colony);
    feedback(
        host,
        format!(
            "{}: job {}, colony {} ({} crops harvested), farmer's hut {}.",
            citizen.name,
            citizen.job.label(),
            colony.name,
            colony.harvested,
            if hut { "present" } else { "missing" }
        ),
    );
}

fn assign(host: &mut dyn ServerHost, player: PlayerRef, event: &RunModCommandEvent<'_>) {
    let Some((hall, mut colony)) = player_colony(host, player) else {
        no_colony(host);
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
            format!("This slice has one job: farmer. {job_id} is not ported."),
        );
        return;
    };
    if job == Job::Farmer && !has_farmers_hut(host, &colony) {
        feedback(
            host,
            "A farmer needs a farmer's hut in the colony: place minecolonies:blockhutfarmer."
                .to_owned(),
        );
        return;
    }
    let who_id = colony.citizens[index].id.clone();
    if colony.assign(&who_id, job).is_err() {
        return;
    }
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
        "Colony {} (id {}, owner {}): centre {}, {} blocks ({} chunks), {} citizens, {} crops harvested, founded at tick {}.",
        colony.name,
        colony.id,
        colony.owner_name,
        colony.center,
        colony.radius(),
        colony.size_chunks,
        colony.citizens.len(),
        colony.harvested,
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
