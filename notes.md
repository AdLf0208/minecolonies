# MineColonies — slices 1–2: town hall → colony → citizens → farmer, lumberjack, miner

**Class:** `content` · **Tier:** `deep` · **Role:** core · **Priority:** P0 · in ATM10.
Ported from **MineColonies** `1.1.1411-1.21.1-snapshot`, jar
`ports/_shared/licence-cache/minecolonies-1.1.1411-1.21.1-snapshot.jar`
(sha256 `cf396583d91d2013e1c364f0b50a2670a4d5459bb3b01bfe78a5933b28f84516`),
upstream `ldtteam/minecolonies` branch `version/main`, commit
`3438c06fc92947b28bbd2fda29f00e207b967a22`.

**Format (2026-11-18):** the flat **SDK package** (mod API `0.2`), version **0.2.4** — one
`.hmod` carrying `horizon-mod.toml`, `src/`, `assets/`, `data/` and the licence files. Pin:
Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20), horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, SDK surface `e0563779…`, package format 6.
The retired `.hnmod` (0.1.1, submission **#951**) is archived under
`ports/_shared/evidence/minecolonies/legacy/`; §"Migration to the SDK format" is the recipe.

## Licence — both halves, read from the artefacts

- **Code: GPL-3.0 (rank 1, copyleft → publishable under the same licence with its source).**
  `META-INF/neoforge.mods.toml` declares `license="GPL3"`; the repository's `LICENSE` is the GNU
  GPL v3 text (35,121 B, sha256 `e1c0ad72…b48704`), shipped as this port's `LICENSE` **inside the
  package** (the packer includes the port root's licence files: `LICENSE` and
  `LICENSE-NOTICE.md` are entries 3 and 4 of `minecolonies.hmod`).
  `LICENSE-NOTICE.md` names the authors (LDT Team), the upstream URL and the jar's sha256, and
  states that this is a derivative under the same licence with its source published, adding no
  further restriction. Its "corresponding source" paragraph now names `src/`, `assets/`, `data/`
  and `tools/` rather than `mixins/minecolonies/` and `content/`.
- **Assets: All Rights Reserved → none copied.** `assets/minecolonies/LICENSE` in the jar is
  19 bytes, `All Rights Reserved`. **Not one file from `assets/minecolonies/**` is in this tree**
  — every texture and model is produced by `tools/gen_assets.py` from the palette below, and the
  lang file is written from scratch. The owner's ruling (ARCHITECTURE §8, 2026-11-12) is what makes
  this port possible: mechanics ported, art regenerated. `python3 tools/gen_assets.py --check`
  still re-renders all 15 asset files **byte-identically** after the move from `content/assets/`
  to `assets/` (the palette and the stored-deflate PNG writer are unchanged).

## The slice

A **town hall founds a colony** (name, owner, centre, size, founded tick and roster saved with the
hall), **a citizen joins it** and takes **one job — farmer — that ticks in the world** (it walks to
a mature crop inside the colony, harvests it and leaves it replanted, counting the crop), and
`/minecolonies colony|citizen|job` is the configuration surface. The colony and the citizen's job
survive `quit` and a relaunch. Chosen because it is the mod's core and provable in the harness:
`docs/modpacks.md`'s `deep` tier is "entities and AI, block entities, new hooks", and this slice is
exactly one block entity, one entity and one tick.

### Slice 2 — `lumberjack` and `miner` (0.2.2 → 0.2.4)

The gap line this slice comes from (`## Gaps`, as it stood before it):

> The other ~40 jobs (builder, miner, guard, crafter, …), the builder and its schematics
> (`blueprints/**`), …

Two more jobs, chosen because they differ in **target selection and world action** — a tree to fell
and replant, a shaft to sink and collect — so the roster tick, the assignment surface and the
colony record generalise instead of being a farmer special case. Not the builder: its
`blueprints/**` are jar content and this port may not carry art, and it needs a schematic format
this slice does not have. No renderer, no entity model, no new SDK hook.

Required behaviour, from upstream's own data and its job rules (implemented, not copied):

| Rule | Source |
|---|---|
| The jobs' ids are `lumberjack` and `miner` | `assets/minecolonies/lang/en_us.json`: `com.minecolonies.job.lumberjack` (labelled *Forester*), `com.minecolonies.job.miner` |
| A lumberjack "cuts down trees and replants them"; a miner "mines out a shaft system under the colony, gathering blocks and ores" | `minecolonies:lumberjack.job.desc`, `minecolonies:miner.job.desc` (same file) |
| One work step per **40** ticks at skill 0, the farmer's `max(1, 40 - skill / 2)` shape | `EntityAIWorkFarmer.java:85,692`; every job at skill 0 in this slice |
| A worker acts on what it can reach without another step (2.5 blocks) and walks 0.6 per step | the farmer's own reach and stride, `runtime::FARMER_REACH`/`STRIDE` |
| A tree is its **connected log cluster**; leaves are not logs | `minecraft:logs` at 26.3 is `*_log`/`*_wood`/`*_stem`/`*_hyphae` (`block-tags.json`); the SDK has no block-tag query, so the id is the rule |
| The stump's soil gets a sapling back | "replants them"; a stripped trunk replants its family, the Nether's stems grow fungi |
| Bedrock is never mined, a shaft is never opened into water or lava, and a miner never works outside its colony | "a dangerous job": the shaft is the colony's, `Level.destroyBlock` is only called on a legal block (`runtime::miner_next`) |

**TDD, in the order the ruling asks for.** The rules live in `runtime.rs` (pure), so the failing
tests came first — 17 of them, with the three new rules as `unimplemented!()` bodies:

```
$ cargo test --offline
not implemented: the tree's connected cluster
---- runtime::tests::a_tree_is_felled_from_the_top stdout ----
thread 'runtime::tests::a_tree_is_felled_from_the_top' panicked at src/runtime.rs:620:5
...
test result: FAILED. 41 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out
```

(`ports/_shared/evidence/minecolonies/sdk-test-slice2-red.log`; the 14 include the record's two new
counters and the field-set table, which the same red run pins.) Then the rules, the glue and the
in-game assertions; `sdk-test.log` is the green run of the same 55 tests.

**What the two jobs do on one work tick** (`support::lumberjack_work`, `support::miner_work`):

- **Lumberjack.** The nearest log inside the colony within 16 blocks is the seed of
  `runtime::tree_at` (a six-neighbour flood fill of the same block, capped at 64 logs and a 32/8
  vertical window). It walks to the next log and fells the **highest** one, so the trunk is never
  left hanging over the cut; the drops go into the colony's chest at the level's `level_tick_post`
  (a block-entity write inside a tick is overwritten by that entity's own write-back), and the
  colony counts each log. When the last log falls it takes **one sapling of that tree** from the
  chest and plants it on the stump's soil — only if the soil grows one and the cell is free. With
  no sapling in the chest the soil is left bare.
- **Miner.** `runtime::miner_next` scans the one column under the miner from its feet down to the
  level's floor: the first non-air block is the next to dig, so a shaft already dug is not re-dug
  and no state has to be saved. It stops at `minecraft:bedrock`, at a fluid, at the world floor and
  outside the colony. Each block is `LevelRef::destroy_block(drop, collect)` — the level's own drop
  path — and its stacks go to the chest, counted as "blocks mined".

**The harness's two assertions** (`tools/e2e.sh` L and M, at `0a875da`):

```
== L: a lumberjack fells whole trees, leaves the leaves, and replants from the chest ==
  the unemployed citizen left the tree whole (4 logs and the leaves stand)
  Bo is now the colony's lumberjack.
  after the fell: soil at 2004,101,-1 is 'minecraft:dirt', stump is 'minecraft:oak_sapling',
    chest [{"count":4,"item":"minecraft:oak_log"}]
  the second tree fell too, its soil bare (no sapling in the chest), 8 logs in all
== M: a miner sinks a 1x1 shaft to the bedrock and collects every drop ==
  the unemployed citizen left the 8-block column whole
  Cass is now the colony's miner.
  the shaft is 1x1 and 8 deep, the bedrock untouched, 8 cobblestone in the chest
```

Both phases carry their own counterfactual **before** the assignment (an unemployed citizen beside
the same tree / column, which stays whole), and the miner's depth is inverted by `SELFTEST=1`
(expecting 9 blocks over the bedrock, so a miner one block short fails). The lumberjack's sapling
rule is proved **both ways** in one run: tree 1 with a sapling in the chest is replanted, tree 2
without one is left bare.

### Behaviour, and where it was read

| Rule | Source |
|---|---|
| Founding distance `max(minColonyDistance=8, initialColonySize=4) × 16` = **128 blocks**, refused under it | `ColonyManager.java:311-320` |
| Distance checked **before** "you already own a colony" | `CreateColonyMessage.java:165` then `:172` |
| Colony size is a square chunk range; initial **4** chunks (64 blocks), max **20** | `ChunkDataHelper.java:236-248`; `ServerConfiguration.java:164-166` |
| Default name pattern `"%s's Colony"` | `manual_en_us.json:547` |
| Rename field: **25** characters, an empty name is not applied | `windowtownhallnameentry.xml`; `WindowTownHallNameEntry.java:45-50` |
| Hut blocks: hardness 10, blast resistance infinite, wood sound, no occlusion | `AbstractColonyBlock.java:70,75,99` |
| Farmer: one work step per **40** ticks at skill 0, harvests a mature crop (`CropBlock.isMaxAge`) | `EntityAIWorkFarmer.java:85,661-676,692,797` |
| A job needs its building (a farmer needs the farmer's hut) | `JobFarmer`/`BuildingFarmer`, `ModBuildings` |
| Citizen base health 20; full saturation 60; the entity carries its colony id and a custom name | `CitizenConstants.java:84`; `ICitizenData.java:30`; `CitizenData.java:552-560` |
| One colony per player | `CreateColonyMessage.java:172-183` |
| Command roots `minecolonies` (alias `mc`), subcommands `colony` and `citizens` | `EntryPoint.java:86-132` |

## Generated art (the palette and the templates)

`tools/gen_assets.py` is the only producer of `assets/minecolonies/**`; `--check` re-renders and
compares byte for byte (the PNGs use stored deflate blocks, so they are identical on any machine).
Palette — one named colour per family, nothing else:

| Name | RGB | Used by |
|---|---|---|
| `town_hall_timber` | `#9a7b52` | town hall texture, walls |
| `town_hall_roof` | `#5d402c` | town hall texture, top four rows |
| `town_hall_window` | `#8cc7d8` | town hall texture, one lit window |
| `farmer_timber` | `#8b6d3f` | farmer's hut texture, walls |
| `farmer_roof` | `#6b8f3a` | farmer's hut texture, top three rows |
| `farmer_door` | `#4a331e` | farmer's hut texture, door |
| `scythe_blade` | `#c8d2da` | scythe sprite, blade |
| `scythe_handle` | `#7a522e` | scythe sprite, shaft |
| `citizen_skin` | `#d9a47a` | citizen texture, upper half |
| `citizen_shirt` | `#3f6f8f` | citizen texture, lower half |

Model templates (every model JSON is one of the three): `cube` (`minecraft:block/cube_all`) for the
town hall, `slab` (one box `[0,0,0]..[16,8,16]`) for the farmer's hut, `cross`
(`minecraft:block/cross`) for the scythe. `tools/assemble_content.py` writes the declarations
(`horizon/block`, `horizon/item`, `horizon/entity_type`, recipes, loot, tags, the creative tab) and
its `--check` proves `data/` is exactly what it would write.

The citizen texture (`textures/entity/citizen.png`, 16×16) is generated but **not yet bound** to a
renderer: the citizen draws with its villager template's model (a gap, below).

## The shape on Horizon (SDK 0.2)

- **Block entities**: `Registrar::block_entity_type("town_hall").blocks(&["blockhuttownhall"])` and
  `…("farmer_hut").blocks(&["blockhutfarmer"])` in `Hooks::register`. The declaration is what
  creates the block entity; there is no per-block-entity moment left to implement (the retired
  `block_entity` helper only returned an empty compound).
- **The colony** is one **attachment** per town hall block entity (`support::COLONY`,
  `Holder::BlockEntity(pos)`), saved through its codec — whose `encode`/`decode` are the runtime's
  own field mapping (`colony_fields` / `colony_from_fields`), the same functions the retired bundle
  wrote into the block entity's `Colony` tag. The farmer's hut is one marker attachment
  (`support::HUT`). `AttachmentQueries::query` lists the dimension's halls — the SDK has **no**
  enumeration of a world's block entities, and a query is its sanctioned replacement.
- **Placement**: `Hooks::entity_place` (`BlockEvent.EntityPlaceEvent`, cancel) refuses an illegal
  founding before anything is written; a legal one is queued and completed at that level's
  `Hooks::level_tick_post`, one tick later, when the block exists (a level-scoped hook, because the
  write goes through that level's attachment store).
- **Entity tick**: `EntityPart::general_entity_tick` on `minecolonies:citizen` (`src/parts.rs`) —
  the citizen's job is one entity type's moment, not a global hook.
- **Commands**: `Hooks::register_commands` pushes `Command::new("<usage>")` usages;
  `Hooks::run_mod_command` receives the **parsed** invocation (literals + typed arguments) instead
  of a raw line.

## Migration to the SDK format

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod` on 2026-11-17. This port was
migrated on 2026-11-18 from pin `9f42064` to `0b14f8a`; the old bundle is archived at
`ports/_shared/evidence/minecolonies/legacy/`.

### File map

| 0.1.1 (`.hnmod`) | 0.2.0 (`.hmod`) | note |
|---|---|---|
| `mixins/minecolonies/runtime.rs` | `src/runtime.rs` | **byte-identical**, sha256 `cca59b39…299e` |
| `mixins/minecolonies/support.rs` (`[[module]] state`) | `src/support.rs` | plain module of the mod crate; glue rewritten for the typed host |
| `mixins/minecolonies/{block_entity,use_item_on,place_block,entity_tick_listened,entity_tick_post,register_commands,perform_command}.rs` | `src/lib.rs` (+ `src/parts.rs`) | one method each — table below |
| `mixins/minecolonies/embedded_content.rs` | — | retired: a flat package carries its own content |
| `mixins/minecolonies/horizon-mixin.toml` (8 injections, schema 6) | `horizon-mod.toml` (`api = "0.2"`, `[native]`) | the manifest is the identity **and** the pin |
| `mixins/minecolonies/data/minecolonies-content.hmod` | `assets/minecolonies/**`, `data/**` | unpacked to the package root |
| `content/horizon-mod.toml` | `horizon-mod.toml` | one manifest, not two |
| `content/minecolonies.hmod` (8,204 B, embedded) | — | the content is no longer a nested archive |
| `Cargo.toml` (`[lib] path = mixins/…/runtime.rs`) | `Cargo.toml` (`horizon-mod-api` from `.horizon/sdk/`) | the crate root is now `src/lib.rs`; `runtime` is a module |
| `rust-toolchain.toml` (1.95.0) | — | removed by the merge; the SDK names the toolchain |
| `tools/package.py`, `tools/package.sh`, `tools/horizon.py` | — | retired with the container (archived) |
| `tools/gen_assets.py`, `tools/assemble_content.py` | same, paths `content/…` → package root | `--check` still byte-identical |
| `tools/e2e.sh` | same, `--dev-mod "$PORT"`, `CONTROL=1` replaces `EMBEDDED=1` | see "Verification" |

### Old helper → new hook (every helper, none omitted)

| 0.1.1 helper (API, kind) | What it did | 0.2.0 | Status |
|---|---|---|---|
| `block_entity.rs` (API 15, replace) | made the two huts' block entities | `Registrar::block_entity_type(…).blocks(…).build()` ×2 | **kept** — as a registration; the SDK creates a declared type's entity with an empty compound, which is all the helper ever returned, so **no `BlockEntityPart` is needed** |
| `use_item_on.rs` (API 13, HEAD, `claimed`) | refused an illegal founding before the block was set | `Hooks::entity_place` → `Flow::Cancel` | **kept** (`BlockEvent.EntityPlaceEvent` is upstream's own cancel point and stops the whole placement, stack kept) |
| `place_block.rs` (API 14, HEAD, listen) | wrote the colony into the hall's block entity | `Hooks::entity_place` (queue) → `Hooks::level_tick_post` (server) → `COLONY.set(…)` | **kept**, one tick later: `EntityPlaceEvent` fires *before* the write, so the colony cannot be stored inside it; the flush re-checks the block and re-runs the rule |
| `entity_tick_listened.rs` (API 146, collect) | opted in to entity ticks | — | **retired, no gap**: a Part is called for its own type; there is no gate to open |
| `entity_tick_post.rs` (API 146, HEAD, listen) | the citizen's job step | `EntityPart::general_entity_tick` on `minecolonies:citizen` (`src/parts.rs`) | **kept** as a **Part** (the citizen is one entity type's moment) |
| `register_commands.rs` (API 8, collect) | declared `/minecolonies …` usages | `Hooks::register_commands` → `Command::new(usage)` | **kept** |
| `perform_command.rs` (API 8, HEAD, `ran`) | parsed a raw command line | `Hooks::run_mod_command` with `CommandInvocation` | **kept**, and better: literals + typed arguments, no `split_whitespace` |
| `embedded_content.rs` (`content_root` HEAD) | wrote the embedded `.hmod` to `.horizon-cache` and appended it to the dev list (AE2 pattern) | — | **retired, no gap**: the package *is* the content |
| `support.rs` (`[[module]]`) | game glue, state, commands | `src/support.rs` | **kept**, rewritten |
| `runtime.rs` (`[[module]]`) | the pure colony rules | `src/runtime.rs` | **kept byte-identical** |

The five `Hooks` methods plus the one `EntityPart` method are the whole mod surface;
`uses.json` inside the package lists them (`hook:entity_place`, `hook:general_entity_tick`,
`hook:register_commands`, `hook:run_mod_command`, `hook:level_tick_post`, `macro:horizon_part`), which
is the machine-checkable half of this table.

### Manifest mapping

| 0.1.1 | 0.2.0 |
|---|---|
| `horizon-mixin.toml` `[mod] id/version/priority` | `horizon-mod.toml` `id = "minecolonies"`, `version = "0.2.0"` (no `priority`: injections are gone) |
| `[compatibility] horizon_revision` | `[native] horizon = "0b14f8a…"` (exact 40 hex) |
| (mixins revision only in the launcher) | `[native] mixins = "e59da90…"` |
| content's `api = "0.2"`, `side`, `capabilities` | the same three fields on the one manifest |
| `mixins/<id>/data/*.hmod` + `content_root` injection | `assets/`, `data/` at the package root |
| `[[module]]` ×2 | `src/lib.rs`, `src/runtime.rs`, `src/support.rs`, `src/parts.rs` compiled by the launcher |

### Build, check and verify (the ladder that ran)

```sh
# inside a build slot, BUILD_LOCAL=1 (a pure local check: no staging tree is involved)
ports/tools/build-slot.sh run --job sdk-mc --port ports/minecolonies --step test  -- cargo test --offline --locked
ports/tools/build-slot.sh run --job sdk-mc --port ports/minecolonies --step lint  -- cargo clippy --offline --locked --all-targets -- -D warnings
ports/tools/build-slot.sh run --job sdk-mc --port ports/minecolonies --step fmt   -- cargo fmt --check
ports/tools/build-slot.sh run --job sdk-mc --port ports/minecolonies --step build -- cargo build --offline --locked
ports/tools/build-slot.sh run --job sdk-mc --port ports/minecolonies --step check -- \
    sh -c 'cargo horizon-mod build && cargo horizon-mod check target/horizon-mod/minecolonies.hmod && \
           cargo horizon-mod fits target/horizon-mod/minecolonies.hmod'
```

`cargo horizon-mod check <file>` is a **registry** check (identity, pin, format, sizes, signature
state) and does **not** compile the Rust; `cargo horizon-mod check --uses <file>` compiles the
package's source against the SDK, and `cargo build` in the port is the same compile with a normal
cargo output.

### Traps a huge port must know

1. **`check` is not a build.** It validates the package and says so in its own output ("native
   source: unsandboxed; not executed by the registry check"). Compile evidence must come from
   `cargo build` / `cargo clippy` in the port — `check --uses` also compiles.
2. **The `.horizon/sdk/` folder is fetched, never packed** (`.gitignore`d). `cargo horizon-mod
   <anything>` fetches it on a fresh clone; the first offline build then needs the registry
   crates, so one online `cargo fetch` per machine.
3. **A module cannot be the crate root any more.** The old `[lib] path = mixins/…/runtime.rs`
   trick is gone: `src/lib.rs` is the crate root (`#[horizon_mod]` must be there) and `runtime.rs`
   is a plain `mod runtime;`. That is *why* the byte-identity rule is checkable: the file moved
   without a single byte changing.
4. **There is no global block-entity enumeration.** `world.block_entities` was in-crate in the old
   glue; the SDK exposes `attachments()` (by holder) and `attachment_query` (a residency view of
   one key), so the colony had to become an attachment. A port that scans the world must be
   redesigned around a query, not translated.
5. **`server_tick` does not say which level the host holds**, so an attachment written there can
   land on another level's store. The flush is `level_tick_post`, which is told its `LevelRef`;
   a queue entry carries its dimension and is dropped after 200 level ticks that are not its own,
   so a level that never ticks cannot leak it.
6. **`EntityPlaceEvent` is before the write.** `pos`/`state` are known, the block is not there
   yet, so nothing may read the block entity or the block in it. Hence the queue + `level_tick_post`.
7. **Parts are per type, and vanilla families are excluded.** `general_entity_tick` is told "only
   for general entity types"; a Part on `minecraft:villager` never ticks. A content-declared type
   (`minecolonies:citizen`) is registered by the type registry, so its Part does.
8. **`AttachmentType` is a `static` with a `codec`**; `get` inserts a default, `get_existing` does
   not. Always read with `get_existing` and write with `set`, or a read creates state.
9. **The packer includes the port root's `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
   `Cargo.toml`, `Cargo.lock`** and everything under `assets/` and `data/`. Nothing else is
   needed for the licence obligation — but a licence file placed in `target/` would never ship.
10. **`check --uses` compiles for wasm as well as natively.** A `Registrar::entity_part` call is
    native-only (`#[cfg(not(target_arch = "wasm32"))]`), so the Part registration and its module
    are gated; `check --uses` is what catches this, a plain `cargo build` is not.
11. **`check --uses` reports the structs built field by field** (`AttachmentType`,
    `AttachmentCodec`, and a false positive on `Text` from `runtime.rs`): if a later game build
    adds a field to either, that line is the one that breaks. Both are SDK-owned structures, so
    the fix belongs upstream or in a one-line constructor change here.
12. **Reproducible bytes:** two `cargo horizon-mod build` runs give the same 53,767 B /
   `0d014f3f…` package (the packer sorts entries and fixes dates).

## Deviations (deliberate, not defects)

- **One citizen**, not `initialCitizenAmount` (4): the slice proves one recruit deterministically.
- The farmer **replants at age 0** and counts the crop instead of breaking it and replanting from
  an inventory: there is no citizen inventory yet, and a dropped item entity would be the only
  observable difference.
- The citizen is `NoAI` and stepped by its job (0.6 blocks per work tick), not pathfound.
- Colonies are found through **loaded, resident** halls (the attachment query is a residency
  view), so upstream's per-world colony list that also covers unloaded chunks is still not
  reproduced. Slice 1's distance check therefore sees only halls whose chunks are loaded — the
  same limit the old world scan had.
- The farm rule is "a mature `minecraft:wheat` inside the colony, within 6 blocks of the citizen",
  where upstream reads a farm field's crop list; the observable behaviour (walk, harvest, replant)
  is the same at this depth.
- Art placeholders: the farmer's hut is a half-height box and the scythe two crossed planes.
- **New in 0.2.2 (slice 2):** the two new jobs need **no building**. Upstream gives every job a
  building (`JobLumberjack`/`BuildingLumberjack`, `JobMiner`/`BuildingMiner`), but this port has no
  lumberjack's hut or mine block — only the farmer's hut exists — so `Job::building()` answers
  `Some(FARMER_HUT)` for the farmer alone and the two new jobs are assignable without one. The
  rule (and its code path) is in place for when those blocks arrive; adding them is content, not
  behaviour, and their `blueprints/**` are jar art this port may not carry.
- **New in 0.2.2 (slice 2):** a citizen has no inventory of its own, so the **colony's chest is its
  store**: the nearest `minecraft:chest` inside the colony within 16 blocks of the worker is where
  a lumberjack's logs and a miner's cobblestone go, and where the sapling a felled tree is
  replanted with comes from. Upstream delivers to the worker's own building's rack; this slice has
  no building racks, and a chest is the observable store the harness can assert (`ctl container`).
  The drops are applied at `level_tick_post`, as a delta on the chest, because a block-entity
  write made during a tick is overwritten by that entity's own write-back.
- **New in 0.2.2 (slice 2):** the lumberjack fells **top-down**, which upstream does too; the
  miner's shaft depth is **derived from the world** (the first non-air block under its feet) rather
  than saved, so a relaunch resumes at the right depth without state.
- **New in 0.2.2 (slice 2):** "never into water/lava" is decided by the block id
  (`minecraft:water`, `minecraft:lava`, `minecraft:bubble_column`, and any id ending `_water`/
  `_lava`/`:water`/`:lava`), not by a fluid-state query: a mod fluid with another name would be
  treated as diggable. The pin exposes fluid states, but a shaft that stops at the two fluids the
  mod's own era had is what the rule states.
- **New in 0.2.2 (slice 2):** the lumberjack breaks one log per work tick (upstream's tree felling
  is also block by block), so a four-log tree takes four steps; the harness waits for the tree to
  drop rather than for a fixed tick count.
- **New in 0.2.0 (migration):** the colony is saved by an attachment codec rather than as a
  `Colony` compound inside the block entity's own saved data. The record, its field names and the
  mapping are unchanged (`colony_fields`/`colony_from_fields`), and the attachment is stored on
  the hall's block-entity holder, so "the colony belongs to its hall and saves with the chunk" is
  preserved in shape; the *mechanism* differs.
- **New in 0.2.0 (migration):** a citizen that had to fall back to `minecraft:villager` (the
  content type missing) does **not** farm: a Part attached to a vanilla family hears no
  `general_entity_tick`. The fallback exists so a missing content half is visible (the harness
  asserts the entity's type) rather than silent.

## Upstream defects fixed

**The founding distance was measured in 3D.** `ColonyManager.isFarEnoughFromColonies` compares the
town hall with the closest colony using `BlockPosUtil.getDistance`, which is
`sqrt(dx² + dy² + dz²)` (`BlockPosUtil.java:453-460`), while a colony is a horizontal claim: the
chunk claim is a square in x/z (`ChunkDataHelper.java:236-248`), the 2D distance helper sits in the
same class (`BlockPosUtil.getDistanceSquared2D`, `:469`), and the town hall's own "too close" token
is "radius + padding + radius" (`ColonyManager.getMinimumDistanceBetweenTownHalls`, `:568-571`).
With the 3D formula a town hall 120 blocks across and 120 blocks up reads as 170 blocks away, so
upstream lets two colonies overlap on the map. **The port uses the horizontal distance** with the
same 128-block threshold; the e2e proves it — the raised town hall at 120 across / 120 up is
refused, which upstream's formula would allow. Worth an upstream report? Yes, a one-line change
(`getDistanceSquared2D`), but not filed: the brief asks only to say whether it is worth one.

**Data that could never match**: the town hall recipe names `structurize:sceptergold`. Structurize
is MineColonies' parent mod and is not ported here, so the recipe could never match at this pin
(the same rule as `sushigocrafting`'s knife tag). The port's recipes keep the shapes and use
vanilla ingredients (planks + a crafting table; planks + an iron hoe; iron ingots + sticks), which
is recorded here rather than shipped broken.

## Carried data fixed (2026-11-21, the `constructionstick` class)

**The scythe recipe was shadowed by vanilla's iron hoe and could never be crafted.**
`data/minecolonies/recipe/scythe.json` was `["II ", " S ", " S "]` — two iron over a two-stick
shaft — which is **exactly** 26.3's `minecraft:iron_hoe` recipe. Measured in game 2026-11-21: a
correctly built 2×3 grid of two iron ingots and two sticks at the crafting table produced
`{"count":1,"item":"minecraft:iron_hoe","slot":200}` instead of the scythe, so the item the port
ships was unobtainable (`e2e-launcher.log`, phase N's red run in the wave's log). **The port moves
an ingredient, same ingredients and same result**: the blade now has a gap (`["I I"," S "," S "]`),
a shape no vanilla recipe owns, and `tools/craft_check.py` crafts it, asserts the id and count,
shift-clicks it into the inventory, and asserts the grid emptied and the iron was consumed — with
two wrong-ingredient controls that must craft nothing. The town hall and the farmer's hut recipes
were **already** craftable (asserted in the same phase). Not an upstream report: upstream's scythe
is not a crafting-table recipe at all (the jar carries none — the builder's `crafterrecipes` make
tools), so the collision came from this port's own vanilla substitution.

## Coverage — every family this port carries, and what is left

The table is the record the parity rule asks for: a family is `tested` in game through
`horizon-ctl` (phase in `tools/e2e.sh`), `not ported` (upstream has it, this slice does not), or
`blocked(<gap>)`. `python3 ports/tools/check-coverage.py minecolonies` reads it.

| Family | Status | Evidence / reason |
|---|---|---|
| `minecolonies:blockhuttownhall` | tested | A: placed from the held item, founds the colony, `colony info` reads it back; C–E: refused placements leave air; the block and its item share the id |
| `minecolonies:blockhutfarmer` | tested | F: placed; H: `job info` reports it as the farmer's building; the block and its item share the id |
| `minecolonies:blocks/blockhuttownhall` | tested | A: the block's own loot table (the declared shape; the block has no `requires_tool`, so a hand break drops it) |
| `minecolonies:blocks/blockhutfarmer` | tested | F: same declared shape, same tag (`minecraft:mineable/axe`, carried by `data/minecraft/tags/block/mineable/axe.json`) |
| `minecolonies:entities/citizen` | tested | G/M: hired, counted live, and the roster is read back; the citizen entity type is what the hire spawns |
| `minecolonies:scythe` | tested | N: crafted in a real table, read back from the inventory; the recipe is the same id |
| `block_entity` | tested | A/B/K: the town hall's block entity holds the colony attachment, which round-trips and survives a relaunch; F/H: the farmer's hut's marker is found by `has_farmers_hut`. Both types are declared in code (`Registrar::block_entity_type`), not in `data/`, which is why the tool sees no id for them |
| `*` | not ported | everything else the jar carries: the other ~92 blocks and ~236 items, the other ~35 jobs, the research tree (`data:researches`), crafter recipes, colony/visitor/name data, the other 150 loot tables, the recipes the port does not carry, the item/block/entity tags, and the world-gen families (`worldgen:structure`, `worldgen:structure_set`, `worldgen:template_pool`, `worldgen:processor_list`, `tag:worldgen`). The builder's `blueprints/**` are jar **art** and may not be carried at all (`blocked(jar-assets)`). The gap list below is the prose form of this row. |

The commands (`colony info/rename/radius`, `citizen hire/list`, `job assign/info`) are not a family
the tool inventories; every one of them is called and its **parsed** reply asserted in the leg
(A, B, G, H, I, L, M), and an unknown job is refused by name.

## Gaps (stated, not hidden)

Content and behaviour, unchanged from 0.1.1:

- The other ~35 jobs (builder, guard, crafter, cook, …), the builder and its schematics
  (`blueprints/**`), the research tree (`data/minecolonies/researches/**`), the colony/citizen UI,
  happiness, saturation, the request system, raids, permissions and ranks, colony size growth from
  buildings, worldgen colonies, and the colony registry across unloaded chunks.
- The citizen's own model and texture (a native renderer slice); the scythe is a plain item with no
  harvest behaviour of its own (the job does the harvesting).
- The colony banner, the supply camp/ship, every other block of the mod, and the **buildings the new
  jobs would need** (a lumberjack's hut, a mine): the jobs themselves are ported and asserted, the
  two blocks are not (see the deviations).

Introduced or left open by the migration — **closed at 0.2.4 (2026-11-21)**: the game binary at
the campaign pin now exists and the leg ran, so the attachment **codec persistence across a
relaunch** (K), the **block entity holder** for a content-declared block (A/B), the hall Part
driving the roster (G–M) and the launcher compiling `src/` from the package directory (every
launch) are all transcripts, not compile-level claims. What remains open:

- A hut placed **before** this port's attachments existed is invisible to
  `has_farmers_hut` until re-placed (no migration from 0.1.1 worlds; the formats differ anyway).
- The villager-fallback citizen does not farm (above).
- `python3 ports/tools/upstream_ready.py minecolonies --copyleft` reports the pin blocker when the
  tool's hardcoded campaign pin differs from the port's: the SDK nightly is a campaign-level ruling,
  not a port defect.

## Review fixes (2026-11-18) — registry review #1175

| Finding | Fix | Test / proof |
|---|---|---|
| `src/support.rs:268` — the nearest-colony fallback served the management commands, so a nearby stranger could rename, resize, hire into and reassign jobs in another player's colony | management (`rename`, `radius`, `hire`, `assign`) now resolves through `owned_colony` and `runtime::may_manage`; reading a neighbour's colony (`info`, `list`, `job info`) is unchanged | `runtime::tests::only_the_owner_may_manage_a_colony` (watched failing with the check removed) |
| `src/support.rs:731` — hire had no roster limit, cooldown or cost and spawned a persistent citizen each time | `runtime::MAX_CITIZENS` (16) with `hire_refusal`, checked **before** a mob is spawned, so both the saved roster and the live mob count are bounded (upstream bounds them by housing, which this slice does not model) | `runtime::tests::a_hire_is_refused_once_the_roster_is_at_the_cap` (watched failing with the refusal removed) |
| `mods/minecolonies.toml:27` — `GPL-3.0-only` vs the port's `-or-later` | the **package** now says `GPL-3.0-only` in `Cargo.toml`, `horizon-mod.toml`, `README.md` and `LICENSE-NOTICE.md`: the jar and the repository say `GPL3`, the licence file has no "or later" clause, and the registry entry's scope is the orchestrator's | the four files quoted in the verification below |

**Found while proving the leg:** the citizen's own `EntityPart` never ticked — a vanilla-family
entity type (the citizen is a villager template) hears no `general_entity_tick` at this pin — so
the job never ran. The town hall now drives its roster: a `TownHallWorker` Part ticks the hall
and steps each citizen on its own 40th tick (unit-tested `uuid_value` round trip). The e2e also
gained a far-arena stand/support and a far-crop assertion that accepts the wheat's regrowth
(age < 7) plus a second harvest count, and the `CONTROL_GAME` control is the Nightly without the
package. **Ladder:** 39 tests, clippy `-D warnings` 0, `fmt --check` clean, two builds
byte-identical, `check` and `fits` exit 0; `launcher-game.sh run minecolonies` → `PASS` +
`EXIT:0`, `GAME=$CONTROL_GAME` → `EXIT:1` (`../_shared/evidence/minecolonies-e2e/`).

Package 0.2.1: **54,038 B**, SHA-256 `5c46f90c4e84c347eae117136235f97bb8c16febe1afd4d7d5a7a898f0d66186`;
a second build is byte-identical (`cmp`), `check` and `fits` exit 0.

## Verification

Skills: superpowers/using-superpowers,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

At **0.2.4 / `0a875da`** (2026-11-21), the logs under `ports/_shared/evidence/minecolonies/` and
`ports/_shared/evidence/minecolonies-e2e/`. Every ladder log prints the sha256 of the delivered
package it was run against.

| step | command | result | log |
|---|---|---|---|
| repin | `cargo horizon-mod update --horizon 0a875da…` | *"MineColonies, from 0b14f8a to 0a875da: none of the 287 items it uses changed."* then *"Pinned to Horizon 0a875da…"*; `.horizon/sdk` refetched (`SDK 0a875da (build 0a875da…)`) | — |
| runtime tests (red) | `cargo test --offline` with the new rules as `unimplemented!()` | **41 passed, 14 failed**, `not implemented: the tree's connected cluster` / `the miner's shaft` / `the lumberjack's step` | `sdk-test-slice2-red.log` |
| runtime tests | `cargo test --offline --locked` | **55 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0, **0 warnings**, `CLIPPY_EXIT:0` | `sdk-clippy.log` |
| format | `cargo fmt --check` | `FMT_EXIT:0` | `sdk-fmt.log` |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2 | **`CMP:identical`**, 63,940 B, sha256 **`d81dcd77b30d37f2c0291ee2b4ee736e03848754d04661f31dc7493960dac0aa`** | `sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log` |
| registry check | `cargo horizon-mod check <delivered file>` | `minecolonies 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 63,940 B, SHA-256 `d81dcd77…ac0aa` | `sdk-check.log` |
| compile the package's source | `cargo horizon-mod check --uses <delivered file>` | exit 0 (the wasm pass; its unused-item notes are the native-only Parts) | `sdk-check-uses.log` |
| fits the pin | `cargo horizon-mod fits <delivered file> --build 0a875da` | `0a875da  made for` | `sdk-fits.log` |
| fits another build | `cargo horizon-mod fits <delivered file> --build 0b14f8a` | `0b14f8a  fits` | `sdk-fits-other.log` |
| content | `python3 tools/gen_assets.py --check`, `python3 tools/assemble_content.py --check` | `assets/minecolonies/: 15 files, byte-identical to the palette's output`; `data/` matches the assembler | `sdk-assets.log` |
| package contents | `unzip -l` | **37 entries**; `LICENSE` (GPL-3.0 text) and `LICENSE-NOTICE.md` inside; **no `Cargo.lock`** | `sdk-entries.txt` |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** | — |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` (the manifest's version moved, so the launcher could not reuse a build) | — |
| in game | `build-slot.sh run … --step e2e -- tools/e2e.sh` | **PASS** + `EXIT:0`, phases A–N | `minecolonies-e2e/e2e-launcher.log` |
| control (the unmodded Nightly) | `GAME=$CONTROL_GAME tools/e2e.sh` | **FAIL** at `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` | `minecolonies-e2e/e2e-launcher-control.log` |
| control inside the same run | `CONTROL=1 tools/e2e.sh` | the control leg says *"without the package: no town hall item"*, then the modded leg is **PASS** + `EXIT:0` | `minecolonies-e2e/e2e-launcher-control-inside.log` |
| fail-capability (depth) | `SELFTEST=1 tools/e2e.sh` | **FAIL** `the miner did not dig 9 blocks above the bedrock`, `EXIT:1` | `minecolonies-e2e/e2e-selftest.log` |
| fail-capability (founding) | `FOUNDING_SELFTEST=1 tools/e2e.sh` | **FAIL** `selftest: the second town hall was refused, as it should be`, `EXIT:1` | `minecolonies-e2e/e2e-founding-selftest.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=5 ok=5 missing-exit=0 bad-exit=0`, exit 0 | — |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | every carried family `tested` or declared `not ported`/`blocked(...)` | — |

**The in-game run at this pin, 2026-11-21** (`minecolonies-e2e/e2e-launcher.log`, the phases in
order): A–E founding and both refusals (including 120 across × 200 up, the fixed defect), F/ G/ H
the hut, the hire and the building rule, I the farmer harvests and walks, J a citizen with no
colony is inert, K the colony survives quit + relaunch, L the two trees (a sapling replanted only
when the chest held one, the leaves untouched, 4 then 8 logs in the chest, `0 → 4 → 8` logs
felled), M the miner's 1×1 eight-block shaft to the bedrock (bedrock and the four neighbouring
floor blocks untouched, 8 cobblestone in the chest, the dug counter 8), N the three recipes crafted
in a real table with their controls, the scythe read back and an unknown id refused. Every
assertion of L and M has its counterfactual in the same run (the citizen unemployed beside the same
tree/column stays whole) and the depth inversion is the `SELFTEST=1` leg.

`tools/e2e.sh` launches with **`--offscreen`** rather than `--headless`: phase N opens a crafting
table's menu and clicks its slots, and a menu needs a screen.

**Not carried, not claimed:** the two jobs' buildings (the deviations section), the builder's
schematics, and everything in the gap list above. The e2e's citizen inventory is the colony's chest,
stated as a deviation, not as upstream's own building rack.

History — the 0.2.1/0.2.0 evidence (`retired-0.2.1/`, moved when 0.2.4 superseded it):

- **0.2.0 / `0b14f8a`** (2026-11-18): 36 tests, clippy 0, `53,767 B` sha256 `0d014f3f…`, `fits`
  `0b14f8a made for`; **no in-game run at that pin** (no game binary existed), recorded as a gap.
- **0.2.1 / `0b14f8a`** (2026-11-19): 39 tests after the registry review's three fixes, `54,038 B`
  sha256 `5c46f90c…`, in-game `PASS` + `EXIT:0` with the control at `EXIT:1`
  (`minecolonies-e2e/e2e-launcher.log`, recorded then; superseded by this run).
- **0.1.1 / `9f42064`**: the retired `.hnmod`; 36 runtime tests first (0 passed / 36 failed against
  `unimplemented!()`), the embedded-content leg with its control, and the 38,828 B bundle
  (`e2e-dev.log`, `e2e-embedded.log`, `e2e-nightly.log`, `e2e-selftest.log`).

## Changelog for the next version (verbatim)

**0.2.4** — the second slice, `lumberjack` and `miner`:

- **`lumberjack`**: a citizen fells a whole connected log cluster inside the colony, top-down, one
  block per work tick, leaving the leaves; the logs go into the colony's chest and the colony counts
  them; the last log puts one sapling of that tree back on the stump's soil **when the chest holds
  one** (a Nether stem replants its fungus). Assigned with
  `/minecolonies job assign <citizen> lumberjack`.
- **`miner`**: a citizen sinks a **1×1 shaft** straight down from where it stands, one block per
  work tick, never into bedrock, never into water or lava, never past the world floor or outside the
  colony, and every drop goes into the colony's chest. Assigned with `… miner`.
- The colony record gained two counters (`Felled`, `Dug`), optional on read so a 0.2.1 colony still
  loads; `/minecolonies colony info` and `job info` print them; `job assign` refuses an unknown job
  by name; both jobs step on the roster's own 40-tick cadence.
- The **scythe recipe** was shadowed by vanilla's iron hoe and could not be crafted; its ingredients
  were re-placed in a shape no vanilla recipe owns, and all three carried recipes are now crafted in
  game (`tools/craft_check.py`).
- Re-pinned to Horizon **`0a875da`** (Nightly 2026-11-20) + horizon-mixins `e59da90`; every in-game
  claim is a transcript at that pin, with a control that fails without the package and two
  fail-capable legs.

Supersedes #951, closed because Horizon retired `.hnmod` (#17992). The `.hnmod` at 0.1.1 and the SDK
package at 0.2.0/0.2.1 stay in the record; this is the 0.2.4 rebuild of the same port with two more
jobs.
