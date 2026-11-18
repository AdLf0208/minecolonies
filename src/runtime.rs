// MineColonies slice 1: the colony rules — founding, radius, the citizen roster, the farmer
// job's step and the colony record's fields.
//
// Pure: no world, no game types, no I/O. `cargo test` exercises it without staging Horizon,
// and the same source is injected into the game as `crate::game::minecolonies`.
//
// Every constant carries the upstream place it was read from (`../../FINDINGS.md` has the
// table).

use std::collections::BTreeMap;
use std::fmt;

/// Blocks in a chunk; every colony distance is counted in chunks upstream.
pub const BLOCKS_PER_CHUNK: i32 = 16;
/// `ServerConfiguration.minColonyDistance` default 8 (`ServerConfiguration.java:165`).
pub const MIN_COLONY_DISTANCE_CHUNKS: i32 = 8;
/// `ServerConfiguration.initialColonySize` default 4 (`ServerConfiguration.java:166`).
pub const INITIAL_COLONY_SIZE_CHUNKS: i32 = 4;
/// `ServerConfiguration.maxColonySize` default 20 (`ServerConfiguration.java:164`).
pub const MAX_COLONY_SIZE_CHUNKS: i32 = 20;
/// The rename field's `maxlength` (`windowtownhallnameentry.xml`).
pub const MAX_NAME_CHARS: usize = 25;
/// `EntityAIWorkFarmer.DEFAULT_DELAY`, at skill 0 (`EntityAIWorkFarmer.java:85`).
pub const FARMER_DELAY_TICKS: u64 = 40;
/// A worker harvests what it can reach without taking another step.
pub const FARMER_REACH: f64 = 2.5;
/// The farmer's stride per work tick, in blocks: a walk, not a teleport across the field.
pub const FARMER_STRIDE: f64 = 0.6;

/// The town hall block (`BlockHutTownHall.getHutName`, `blockhuttownhall`).
pub const TOWN_HALL: &str = "minecolonies:blockhuttownhall";
/// The farmer's hut block: a farmer works only where the colony has its building.
pub const FARMER_HUT: &str = "minecolonies:blockhutfarmer";
/// The citizen's content entity type.
pub const CITIZEN: &str = "minecolonies:citizen";
/// The crop the farmer works (`Blocks.WHEAT`).
pub const WHEAT: &str = "minecraft:wheat";
/// `CropBlock.isMaxAge` for wheat (`Age 7`).
pub const WHEAT_MAX_AGE: &str = "7";
/// The age a harvested wheat is left at: the farmer's replant.
pub const WHEAT_REPLANT_AGE: &str = "0";

/// A block position. The colony is horizontal, so `y` is carried but never compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Pos {
    /// A position.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

/// The player who founds a colony: their UUID (`Permissions.setOwner`) and name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub id: String,
    pub name: String,
}

/// What the founding check knows about a colony that already exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownHall {
    pub id: u32,
    pub name: String,
    pub owner: String,
    pub center: Pos,
}

/// Why founding was refused. Each variant is one of the mod's own refusals.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The action had no player behind it (no owner to give the colony).
    NoOwner,
    /// The name is empty once trimmed (`WindowTownHallNameEntry.java:47`).
    EmptyName,
    /// The name is longer than the rename field allows, or holds a control character.
    BadName,
    /// Too close to a colony: the mod refuses under 128 blocks (`ColonyManager.java:313-320`).
    TooClose { colony: String, distance: f64 },
    /// The player already owns a colony (`CreateColonyMessage.java:172-183`).
    AlreadyOwned { colony: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOwner => write!(f, "a colony needs a player to own it"),
            Self::EmptyName => write!(f, "a colony needs a name"),
            Self::BadName => write!(f, "a colony name is 1 to {MAX_NAME_CHARS} characters"),
            Self::TooClose { colony, distance } => write!(
                f,
                "too close to {colony}: {distance:.0} blocks, the closest a colony may stand is {MIN_COLONY_DISTANCE_CHUNKS} chunks"
            ),
            Self::AlreadyOwned { colony } => write!(f, "you already own {colony}"),
        }
    }
}

impl std::error::Error for Refusal {}

/// A colony about to be founded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Founding {
    pub id: u32,
    pub name: String,
    pub owner: Owner,
    pub center: Pos,
    pub size_chunks: i32,
}

/// Trims and checks a colony name: 1 to 25 characters, no control characters.
///
/// # Errors
/// `EmptyName` for nothing left after trimming, `BadName` for more than 25 characters or a
/// control character.
pub fn validate_name(raw: &str) -> Result<String, Refusal> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(Refusal::EmptyName);
    }
    if name.chars().count() > MAX_NAME_CHARS || name.chars().any(char::is_control) {
        return Err(Refusal::BadName);
    }
    Ok(name.to_owned())
}

/// The name a new colony gets before it is renamed: upstream's `"%s's Colony"` pattern
/// (`manual_en_us.json:547`).
#[must_use]
pub fn default_name(owner: &str) -> String {
    format!("{owner}'s Colony")
}

/// The mod's closest a colony may stand, in blocks (`ColonyManager.java:313`).
#[must_use]
pub fn founding_distance_blocks() -> i32 {
    MIN_COLONY_DISTANCE_CHUNKS.max(INITIAL_COLONY_SIZE_CHUNKS) * BLOCKS_PER_CHUNK
}

/// The horizontal distance between two positions: the colony is a 2D claim.
#[must_use]
pub fn horizontal_distance(a: Pos, b: Pos) -> f64 {
    let dx = f64::from(a.x - b.x);
    let dz = f64::from(a.z - b.z);
    dx.hypot(dz)
}

/// The colony's radius in blocks: `size_chunks * 16` (`ChunkDataHelper.java:236-248`).
#[must_use]
pub fn radius_blocks(size_chunks: i32) -> i32 {
    size_chunks * BLOCKS_PER_CHUNK
}

/// Whether `pos` is inside the colony: a square of `radius_blocks`, any height.
#[must_use]
pub fn in_colony(center: Pos, size_chunks: i32, pos: Pos) -> bool {
    let radius = radius_blocks(size_chunks);
    (pos.x - center.x).abs() <= radius && (pos.z - center.z).abs() <= radius
}

/// `maxColonySize` clamps an asked-for size (`ServerConfiguration.java:164`).
#[must_use]
pub fn clamp_size(chunks: i32) -> i32 {
    chunks.clamp(1, MAX_COLONY_SIZE_CHUNKS)
}

/// The next free colony id: one past the highest.
#[must_use]
pub fn next_id(existing: &[TownHall]) -> u32 {
    existing.iter().map(|hall| hall.id).max().unwrap_or(0) + 1
}

/// The founding check, in upstream's order: the name, the owner, the distance
/// (`CreateColonyMessage.java:165`) and then one colony per owner (`:172`).
///
/// # Errors
/// The first `Refusal` that applies.
pub fn founding_refusal(
    name: &str,
    owner: Option<&Owner>,
    center: Pos,
    existing: &[TownHall],
) -> Result<Founding, Refusal> {
    let name = validate_name(name)?;
    let owner = owner.ok_or(Refusal::NoOwner)?;
    let range = f64::from(founding_distance_blocks());
    // `CreateColonyMessage.java:165`: the distance is checked before the owner.
    if let Some((closest, distance)) = existing
        .iter()
        .map(|hall| (hall, horizontal_distance(center, hall.center)))
        .filter(|(_, distance)| *distance < range)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
    {
        return Err(Refusal::TooClose {
            colony: closest.name.clone(),
            distance,
        });
    }
    // `CreateColonyMessage.java:172-183`: one colony per player.
    if let Some(mine) = existing.iter().find(|hall| hall.owner == owner.id) {
        return Err(Refusal::AlreadyOwned {
            colony: mine.name.clone(),
        });
    }
    Ok(Founding {
        id: next_id(existing),
        name,
        owner: owner.clone(),
        center,
        size_chunks: INITIAL_COLONY_SIZE_CHUNKS,
    })
}

/// A job a citizen can hold. Slice 1 implements the farmer; the rest are the mod's own jobs,
/// listed as a gap in `notes.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    Unemployed,
    Farmer,
}

impl Job {
    /// The id the entity tag and the colony record store.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Unemployed => "unemployed",
            Self::Farmer => "farmer",
        }
    }

    /// The id back, or None for one this slice does not implement.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "unemployed" => Some(Self::Unemployed),
            "farmer" => Some(Self::Farmer),
            _ => None,
        }
    }

    /// The name a line of feedback uses.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unemployed => "Unemployed",
            Self::Farmer => "Farmer",
        }
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// One citizen of the roster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citizen {
    pub id: String,
    pub name: String,
    pub job: Job,
}

/// The colony record, as the town hall block entity saves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colony {
    pub id: u32,
    pub name: String,
    pub owner: String,
    pub owner_name: String,
    pub center: Pos,
    pub size_chunks: i32,
    /// The game tick the colony was founded (`Level.getGameTime`).
    pub founded: i64,
    /// Crops the colony's workers have harvested.
    pub harvested: u64,
    pub citizens: Vec<Citizen>,
}

impl Colony {
    /// The colony the founding produced, with no citizens yet.
    #[must_use]
    pub fn founded(founding: Founding, now: i64) -> Self {
        Self {
            id: founding.id,
            name: founding.name,
            owner: founding.owner.id,
            owner_name: founding.owner.name,
            center: founding.center,
            size_chunks: founding.size_chunks,
            founded: now,
            harvested: 0,
            citizens: Vec::new(),
        }
    }

    /// The radius in blocks.
    #[must_use]
    pub fn radius(&self) -> i32 {
        radius_blocks(self.size_chunks)
    }

    /// Whether `pos` is inside the colony.
    #[must_use]
    pub fn contains(&self, pos: Pos) -> bool {
        in_colony(self.center, self.size_chunks, pos)
    }

    /// The citizen with `id`, if the roster has one.
    #[must_use]
    pub fn citizen(&self, id: &str) -> Option<&Citizen> {
        self.citizens.iter().find(|citizen| citizen.id == id)
    }

    /// Gives citizen `id` job `job`.
    ///
    /// # Errors
    /// `UnknownCitizen` when the roster has no such id.
    pub fn assign(&mut self, id: &str, job: Job) -> Result<(), RosterError> {
        match self.citizens.iter_mut().find(|citizen| citizen.id == id) {
            Some(citizen) => {
                citizen.job = job;
                Ok(())
            }
            None => Err(RosterError::UnknownCitizen(id.to_owned())),
        }
    }

    /// Adds a citizen.
    pub fn hire(&mut self, citizen: Citizen) {
        self.citizens.push(citizen);
    }

    /// The harvested count after one crop.
    pub fn harvest(&mut self) {
        self.harvested += 1;
    }
}

/// Why a roster change was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RosterError {
    UnknownCitizen(String),
}

impl fmt::Display for RosterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCitizen(id) => write!(f, "no citizen {id}"),
        }
    }
}

impl std::error::Error for RosterError {}

/// The entity tags that link a citizen to its colony and its job (API 386: tags are saved).
#[must_use]
pub fn colony_tag(colony_id: u32) -> String {
    format!("{COLONY_TAG_PREFIX}{colony_id}")
}

/// The tag prefix a citizen carries for its colony.
pub const COLONY_TAG_PREFIX: &str = "minecolonies.colony=";

/// The colony id in a citizen's tags.
#[must_use]
pub fn read_colony_tag(tags: &[String]) -> Option<u32> {
    tags.iter()
        .find_map(|tag| tag.strip_prefix(COLONY_TAG_PREFIX))
        .and_then(|id| id.parse().ok())
}

/// Whether this work tick belongs to the citizen: one work step every `delay` ticks, the
/// citizens spread across the interval by their entity id.
#[must_use]
pub fn should_work(game_time: i64, entity: u32, delay: u64) -> bool {
    if delay == 0 {
        return false;
    }
    let phase = i64::from(entity) % delay as i64;
    // `rem_euclid` keeps the phase right for a negative or wrapping game time.
    game_time.rem_euclid(delay as i64) == phase
}

/// What the farmer does on one work tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// Walk toward the crop.
    Walk([f64; 3]),
    /// Harvest the crop at the position.
    Harvest(Pos),
}

/// The farmer's step: walk toward the crop while it is farther than `reach`, harvest it
/// otherwise. None when there is nothing to work.
#[must_use]
pub fn farmer_step(citizen: [f64; 3], crop: Option<Pos>, reach: f64) -> Option<Step> {
    let crop = crop?;
    let dx = f64::from(crop.x) + 0.5 - citizen[0];
    let dz = f64::from(crop.z) + 0.5 - citizen[2];
    if dx.hypot(dz) <= reach {
        return Some(Step::Harvest(crop));
    }
    Some(Step::Walk(step_toward(citizen, crop, FARMER_STRIDE)))
}

/// One stride from `from` toward `to`, at most `stride` blocks horizontally, keeping `y`.
#[must_use]
pub fn step_toward(from: [f64; 3], to: Pos, stride: f64) -> [f64; 3] {
    // Aim at the crop block's middle: a block's position is its low corner.
    let dx = f64::from(to.x) + 0.5 - from[0];
    let dz = f64::from(to.z) + 0.5 - from[2];
    let distance = dx.hypot(dz);
    if distance <= stride {
        return [f64::from(to.x) + 0.5, from[1], f64::from(to.z) + 0.5];
    }
    [
        from[0] + dx / distance * stride,
        from[1],
        from[2] + dz / distance * stride,
    ]
}

/// A saved value of the colony record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Long(i64),
    Text(String),
    /// `(id, name, job id)`, as the roster is written out.
    Citizens(Vec<(String, String, String)>),
}

/// Why a saved record could not be read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    Missing(&'static str),
    Kind(&'static str),
    Job(String),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(key) => write!(f, "the colony record has no {key}"),
            Self::Kind(key) => write!(f, "the colony record's {key} has the wrong kind"),
            Self::Job(job) => write!(f, "unknown job {job}"),
        }
    }
}

impl std::error::Error for RecordError {}

/// The record's saved fields: a stable key per value, in one order.
#[must_use]
pub fn colony_fields(colony: &Colony) -> Vec<(&'static str, Value)> {
    vec![
        (
            "Id",
            Value::Int(i32::try_from(colony.id).unwrap_or(i32::MAX)),
        ),
        ("Name", Value::Text(colony.name.clone())),
        ("Owner", Value::Text(colony.owner.clone())),
        ("OwnerName", Value::Text(colony.owner_name.clone())),
        ("Center", Value::Long(pack_pos(colony.center))),
        ("Size", Value::Int(colony.size_chunks)),
        ("Founded", Value::Long(colony.founded)),
        (
            "Harvested",
            Value::Long(i64::try_from(colony.harvested).unwrap_or(i64::MAX)),
        ),
        (
            "Citizens",
            Value::Citizens(
                colony
                    .citizens
                    .iter()
                    .map(|citizen| {
                        (
                            citizen.id.clone(),
                            citizen.name.clone(),
                            citizen.job.id().to_owned(),
                        )
                    })
                    .collect(),
            ),
        ),
    ]
}

/// The record back from its saved fields (the inverse of [`colony_fields`]).
///
/// # Errors
/// `Missing` for a required key, `Job` for a job id this slice does not know.
pub fn colony_from_fields(fields: &[(&str, Value)]) -> Result<Colony, RecordError> {
    let map: BTreeMap<&str, &Value> = fields.iter().map(|(key, value)| (*key, value)).collect();
    let text = |key: &'static str| match map.get(key) {
        Some(Value::Text(text)) => Ok(text.clone()),
        Some(_) => Err(RecordError::Kind(key)),
        None => Err(RecordError::Missing(key)),
    };
    let int = |key: &'static str| match map.get(key) {
        Some(Value::Int(value)) => Ok(*value),
        Some(_) => Err(RecordError::Kind(key)),
        None => Err(RecordError::Missing(key)),
    };
    let long = |key: &'static str| match map.get(key) {
        Some(Value::Long(value)) => Ok(*value),
        Some(_) => Err(RecordError::Kind(key)),
        None => Err(RecordError::Missing(key)),
    };
    let mut citizens = Vec::new();
    match map.get("Citizens") {
        Some(Value::Citizens(rows)) => {
            for (id, name, job) in rows {
                let job = Job::from_id(job).ok_or_else(|| RecordError::Job(job.clone()))?;
                citizens.push(Citizen {
                    id: id.clone(),
                    name: name.clone(),
                    job,
                });
            }
        }
        Some(_) => return Err(RecordError::Kind("Citizens")),
        None => return Err(RecordError::Missing("Citizens")),
    }
    Ok(Colony {
        id: u32::try_from(int("Id")?).map_err(|_| RecordError::Kind("Id"))?,
        name: text("Name")?,
        owner: text("Owner")?,
        owner_name: text("OwnerName")?,
        center: unpack_pos(long("Center")?),
        size_chunks: int("Size")?,
        founded: long("Founded")?,
        harvested: u64::try_from(long("Harvested")?).map_err(|_| RecordError::Kind("Harvested"))?,
        citizens,
    })
}

/// A position packed into one `long`, the colony record's `Center` field.
#[must_use]
pub fn pack_pos(pos: Pos) -> i64 {
    (i64::from(pos.x & 0xFF_FFFF) << 40)
        | (i64::from(pos.y & 0xFFF) << 28)
        | i64::from(pos.z & 0xFF_FFFF)
}

/// A position back from its packed `long`.
#[must_use]
pub fn unpack_pos(packed: i64) -> Pos {
    Pos::new(
        sign_extend(packed >> 40, 24),
        sign_extend(packed >> 28, 12),
        sign_extend(packed, 28),
    )
}

/// Sign-extends the low `bits` of `value` to an `i32`.
fn sign_extend(value: i64, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> Owner {
        Owner {
            id: "00000000-0000-0000-0000-000000000001".to_owned(),
            name: "Builder".to_owned(),
        }
    }

    fn centre() -> Pos {
        Pos::new(200, 101, 0)
    }

    fn existing(id: u32, name: &str, owner: &str, x: i32, z: i32) -> TownHall {
        TownHall {
            id,
            name: name.to_owned(),
            owner: owner.to_owned(),
            center: Pos::new(x, 101, z),
        }
    }

    // --- names -----------------------------------------------------------------------------

    #[test]
    fn a_name_is_trimmed_and_kept() {
        assert_eq!(validate_name("  Riverside "), Ok("Riverside".to_owned()));
    }

    #[test]
    fn an_empty_name_is_refused() {
        assert_eq!(validate_name("   "), Err(Refusal::EmptyName));
    }

    #[test]
    fn a_name_of_25_characters_fits_and_26_do_not() {
        let ok = "a".repeat(MAX_NAME_CHARS);
        assert_eq!(validate_name(&ok), Ok(ok));
        assert_eq!(
            validate_name(&"a".repeat(MAX_NAME_CHARS + 1)),
            Err(Refusal::BadName)
        );
    }

    #[test]
    fn a_name_with_a_control_character_is_refused() {
        assert_eq!(validate_name("two\nlines"), Err(Refusal::BadName));
    }

    #[test]
    fn the_default_name_is_the_owners_pattern() {
        assert_eq!(default_name("Builder"), "Builder's Colony");
    }

    // --- distance and radius ---------------------------------------------------------------

    #[test]
    fn the_founding_distance_is_eight_chunks() {
        assert_eq!(founding_distance_blocks(), 128);
    }

    #[test]
    fn the_distance_is_horizontal_only() {
        let a = Pos::new(0, 64, 0);
        let b = Pos::new(3, 640, 4);
        assert!((horizontal_distance(a, b) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_initial_radius_is_four_chunks() {
        assert_eq!(radius_blocks(INITIAL_COLONY_SIZE_CHUNKS), 64);
        assert_eq!(radius_blocks(0), 0);
    }

    #[test]
    fn the_colony_is_a_square_of_any_height() {
        let c = centre();
        assert!(in_colony(c, 4, Pos::new(264, 101, 0)));
        assert!(in_colony(c, 4, Pos::new(200, -60, 64)));
        assert!(!in_colony(c, 4, Pos::new(265, 101, 0)));
        assert!(!in_colony(c, 4, Pos::new(200, 101, -65)));
    }

    #[test]
    fn the_size_is_clamped_to_the_config() {
        assert_eq!(clamp_size(0), 1);
        assert_eq!(clamp_size(4), 4);
        assert_eq!(clamp_size(99), MAX_COLONY_SIZE_CHUNKS);
    }

    // --- founding --------------------------------------------------------------------------

    #[test]
    fn a_founding_far_from_every_colony_succeeds() {
        let founded = founding_refusal(
            "Riverside",
            Some(&owner()),
            centre(),
            &[existing(1, "Old Town", "someone-else", 0, 0)],
        )
        .expect("the site is 200 blocks from the only colony");
        assert_eq!(founded.id, 2, "the next id follows the highest");
        assert_eq!(founded.name, "Riverside");
        assert_eq!(founded.center, centre());
        assert_eq!(founded.size_chunks, INITIAL_COLONY_SIZE_CHUNKS);
    }

    #[test]
    fn a_colony_without_an_owner_is_refused() {
        assert_eq!(
            founding_refusal("Riverside", None, centre(), &[]),
            Err(Refusal::NoOwner)
        );
    }

    #[test]
    fn a_colony_without_a_name_is_refused_before_the_distance() {
        assert_eq!(
            founding_refusal(
                "  ",
                Some(&owner()),
                centre(),
                &[existing(1, "Old", "x", 200, 0)]
            ),
            Err(Refusal::EmptyName)
        );
    }

    #[test]
    fn a_site_under_128_blocks_from_another_colony_is_refused() {
        let other = existing(7, "Old Town", "someone-else", 200 + 127, 0);
        match founding_refusal("Riverside", Some(&owner()), centre(), &[other]) {
            Err(Refusal::TooClose { colony, distance }) => {
                assert_eq!(colony, "Old Town");
                assert!((distance - 127.0).abs() < 0.001);
            }
            other => panic!("expected TooClose, got {other:?}"),
        }
    }

    #[test]
    fn a_site_exactly_128_blocks_away_is_allowed() {
        let other = existing(7, "Old Town", "someone-else", 200 + 128, 0);
        assert!(
            founding_refusal("Riverside", Some(&owner()), centre(), &[other]).is_ok(),
            "upstream refuses only under the range (ColonyManager.java:317)"
        );
    }

    #[test]
    fn a_hall_far_below_a_colony_is_still_too_close() {
        // Upstream measures 3D (`BlockPosUtil.getDistance`), so 120 blocks across and 200 down
        // reads as 233 blocks and is allowed; the colony claim is 2D, so the port refuses it.
        let other = TownHall {
            id: 7,
            name: "Old Town".to_owned(),
            owner: "someone-else".to_owned(),
            center: Pos::new(320, 301, 0),
        };
        let deep = Pos::new(200, 101, 0);
        assert!(
            matches!(
                founding_refusal("Riverside", Some(&owner()), deep, &[other]),
                Err(Refusal::TooClose { .. })
            ),
            "120 blocks horizontally is under the range whatever the height difference"
        );
    }

    #[test]
    fn the_owner_of_a_colony_cannot_found_another() {
        let mine = existing(3, "Riverside", &owner().id, 900, 900);
        match founding_refusal("Second", Some(&owner()), centre(), &[mine]) {
            Err(Refusal::AlreadyOwned { colony }) => assert_eq!(colony, "Riverside"),
            other => panic!("expected AlreadyOwned, got {other:?}"),
        }
    }

    #[test]
    fn the_distance_is_checked_before_the_owner() {
        let mine = existing(3, "Riverside", &owner().id, 210, 0);
        assert!(matches!(
            founding_refusal("Second", Some(&owner()), centre(), &[mine]),
            Err(Refusal::TooClose { .. })
        ));
    }

    // --- the record ------------------------------------------------------------------------

    fn colony() -> Colony {
        Colony {
            id: 4,
            name: "Riverside".to_owned(),
            owner: owner().id,
            owner_name: owner().name,
            center: centre(),
            size_chunks: 6,
            founded: 1234,
            harvested: 9,
            citizens: vec![
                Citizen {
                    id: "c-1".to_owned(),
                    name: "Ada".to_owned(),
                    job: Job::Farmer,
                },
                Citizen {
                    id: "c-2".to_owned(),
                    name: "Bo".to_owned(),
                    job: Job::Unemployed,
                },
            ],
        }
    }

    #[test]
    fn the_record_survives_a_round_trip() {
        let colony = colony();
        let fields = colony_fields(&colony);
        assert_eq!(colony_from_fields(&fields), Ok(colony));
    }

    #[test]
    fn the_fields_are_a_stable_set() {
        let keys: Vec<&str> = colony_fields(&colony())
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(
            keys,
            [
                "Id",
                "Name",
                "Owner",
                "OwnerName",
                "Center",
                "Size",
                "Founded",
                "Harvested",
                "Citizens"
            ]
        );
    }

    #[test]
    fn a_record_without_a_name_is_refused() {
        let mut fields = colony_fields(&colony());
        fields.retain(|(key, _)| *key != "Name");
        assert_eq!(
            colony_from_fields(&fields),
            Err(RecordError::Missing("Name"))
        );
    }

    #[test]
    fn a_record_with_a_field_of_the_wrong_kind_is_refused() {
        let mut fields = colony_fields(&colony());
        for (key, value) in &mut fields {
            if *key == "Id" {
                *value = Value::Text("four".to_owned());
            }
        }
        assert_eq!(colony_from_fields(&fields), Err(RecordError::Kind("Id")));
    }

    #[test]
    fn a_record_with_an_unknown_job_is_refused() {
        let mut fields = colony_fields(&colony());
        for (key, value) in &mut fields {
            if *key == "Citizens" {
                *value = Value::Citizens(vec![(
                    "c-1".to_owned(),
                    "Ada".to_owned(),
                    "miner".to_owned(),
                )]);
            }
        }
        assert_eq!(
            colony_from_fields(&fields),
            Err(RecordError::Job("miner".to_owned()))
        );
    }

    #[test]
    fn the_roster_is_written_as_job_ids() {
        let fields = colony_fields(&colony());
        let Some((_, Value::Citizens(citizens))) =
            fields.iter().find(|(key, _)| *key == "Citizens")
        else {
            panic!("the record has no citizen list");
        };
        assert_eq!(
            citizens[0],
            ("c-1".to_owned(), "Ada".to_owned(), "farmer".to_owned())
        );
    }

    // --- jobs and citizens -----------------------------------------------------------------

    #[test]
    fn a_job_round_trips_through_its_id() {
        for job in [Job::Unemployed, Job::Farmer] {
            assert_eq!(Job::from_id(job.id()), Some(job));
        }
        assert_eq!(Job::from_id("guard"), None);
    }

    #[test]
    fn the_farmer_is_labelled() {
        assert_eq!(Job::Farmer.to_string(), "Farmer");
    }

    #[test]
    fn hiring_adds_a_citizen_and_assigning_gives_it_the_job() {
        let mut colony = Colony::founded(
            founding_refusal("Riverside", Some(&owner()), centre(), &[]).expect("a free site"),
            7,
        );
        assert!(colony.citizens.is_empty());
        colony.hire(Citizen {
            id: "c-1".to_owned(),
            name: "Ada".to_owned(),
            job: Job::Unemployed,
        });
        assert_eq!(colony.citizen("c-1").map(|c| c.job), Some(Job::Unemployed));
        assert_eq!(colony.assign("c-1", Job::Farmer), Ok(()));
        assert_eq!(colony.citizen("c-1").map(|c| c.job), Some(Job::Farmer));
        assert_eq!(
            colony.assign("c-9", Job::Farmer),
            Err(RosterError::UnknownCitizen("c-9".to_owned()))
        );
    }

    #[test]
    fn a_founded_colony_starts_unworked() {
        let colony = Colony::founded(
            founding_refusal("Riverside", Some(&owner()), centre(), &[]).expect("a free site"),
            7,
        );
        assert_eq!(colony.founded, 7);
        assert_eq!(colony.harvested, 0);
        assert_eq!(colony.size_chunks, INITIAL_COLONY_SIZE_CHUNKS);
    }

    #[test]
    fn a_harvest_counts_once() {
        let mut colony = colony();
        colony.harvest();
        colony.harvest();
        assert_eq!(colony.harvested, 11);
    }

    // --- tags ------------------------------------------------------------------------------

    #[test]
    fn the_colony_tag_names_the_colony() {
        let tags = vec![colony_tag(12), "other".to_owned()];
        assert_eq!(read_colony_tag(&tags), Some(12));
    }

    #[test]
    fn a_citizen_without_a_colony_tag_reads_none() {
        assert_eq!(read_colony_tag(&[]), None);
        assert_eq!(read_colony_tag(&["minecolonies.colony=x".to_owned()]), None);
        assert_eq!(
            read_colony_tag(&["minecolonies.job=farmer".to_owned()]),
            None
        );
    }

    // --- the work tick ---------------------------------------------------------------------

    #[test]
    fn a_citizen_works_once_per_interval_phase() {
        assert!(should_work(40, 0, FARMER_DELAY_TICKS));
        assert!(should_work(41, 1, FARMER_DELAY_TICKS));
        assert!(!should_work(41, 0, FARMER_DELAY_TICKS));
        assert!(!should_work(40, 1, FARMER_DELAY_TICKS));
    }

    #[test]
    fn a_farmer_walks_toward_a_far_crop() {
        let crop = Pos::new(210, 101, 0);
        let step = farmer_step([200.0, 101.0, 0.0], Some(crop), FARMER_REACH);
        match step {
            Some(Step::Walk(to)) => {
                assert!(to[0] > 200.0 && to[0] <= 200.0 + FARMER_STRIDE);
                assert!((to[1] - 101.0).abs() < f64::EPSILON);
            }
            other => panic!("expected a walk, got {other:?}"),
        }
    }

    #[test]
    fn a_farmer_harvests_a_crop_within_reach() {
        let crop = Pos::new(201, 101, 1);
        assert_eq!(
            farmer_step([200.0, 101.0, 0.0], Some(crop), FARMER_REACH),
            Some(Step::Harvest(crop))
        );
    }

    #[test]
    fn a_farmer_with_no_crop_does_nothing() {
        assert_eq!(farmer_step([200.0, 101.0, 0.0], None, FARMER_REACH), None);
    }

    #[test]
    fn a_stride_is_bounded_and_keeps_the_height() {
        let to = step_toward([0.0, 64.0, 0.0], Pos::new(100, 70, 0), 0.6);
        assert!(to[0] > 0.0 && to[0] <= 0.6, "{to:?} is one stride east");
        assert!((to[1] - 64.0).abs() < f64::EPSILON);
        // Aiming at the block's middle stops there instead of overshooting it.
        let to = step_toward([0.0, 64.0, 0.0], Pos::new(0, 0, 0), 5.0);
        assert!(
            (to[0] - 0.5).abs() < 1e-9 && (to[2] - 0.5).abs() < 1e-9,
            "{to:?}"
        );
    }
}
