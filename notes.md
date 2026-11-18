# MineColonies — slice 1: town hall → colony → one citizen → one job

**Class:** `content` · **Tier:** `deep` · **Role:** core · **Priority:** P0 · in ATM10.
Ported from **MineColonies** `1.1.1411-1.21.1-snapshot`, jar
`ports/_shared/licence-cache/minecolonies-1.1.1411-1.21.1-snapshot.jar`
(sha256 `cf396583d91d2013e1c364f0b50a2670a4d5459bb3b01bfe78a5933b28f84516`),
upstream `ldtteam/minecolonies` branch `version/main`, commit
`3438c06fc92947b28bbd2fda29f00e207b967a22`.

**Format (2026-11-18):** the flat **SDK package** (mod API `0.2`), version **0.2.0** — one
`.hmod` carrying `horizon-mod.toml`, `src/`, `assets/`, `data/` and the licence files. Pin:
Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (Nightly 2026-11-18), horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, SDK surface `72aaa95c…`, package format 6.
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

## Gaps (stated, not hidden)

Content and behaviour, unchanged from 0.1.1:

- The other ~40 jobs (builder, miner, guard, crafter, …), the builder and its schematics
  (`blueprints/**`), the research tree (`data/minecolonies/researches/**`), the colony/citizen UI,
  happiness, saturation, the request system, raids, permissions and ranks, colony size growth from
  buildings, worldgen colonies, and the colony registry across unloaded chunks.
- The citizen's own model and texture (a native renderer slice); the scythe is a plain item with no
  harvest behaviour of its own (the job does the harvesting).
- The colony banner, the supply camp/ship, and every other block of the mod.

Introduced or left open by the migration:

- **Not verified in game at this pin.** No game binary exists at `0b14f8a` (the shared pairs are
  `41f822b6` and `1d63e0a5`), so no `horizon-ctl` transcript exists for 0.2.0. Everything below is
  compile-, check- and test-level evidence only: the attachment **codec persistence across a
  relaunch**, the **block entity holder** existing for a content-declared block (the declaration is
  the SDK's mechanism), `general_entity_tick` firing for a **content-declared** entity type, and
  `--dev-mod <package directory>` making the launcher compile `src/` (the e2e's launch form).
- A hut placed **before** this port's attachments existed is invisible to
  `has_farmers_hut` until re-placed (no migration from 0.1.1 worlds; the formats differ anyway).
- The villager-fallback citizen does not farm (above).
- `python3 ports/tools/upstream_ready.py minecolonies --copyleft` now reports the pin blocker
  (`pinned to 0b14f8ac, not 1d63e0a5`): the tool hardcodes the campaign pin, and the SDK nightly is
  a campaign-level ruling, not a port defect.

## Verification

Skills: superpowers/using-superpowers,writing-plans,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

At **0.2.0 / `0b14f8a`** (2026-11-18), every log under
`ports/_shared/evidence/minecolonies/`:

| step | command | result | log |
|---|---|---|---|
| runtime tests | `cargo test --offline --locked` | **36 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0, **0 warnings** | `sdk-lint.log` |
| format | `cargo fmt --check` | `fmt_exit=0` | `sdk-fmt.log` |
| compile | `cargo build --offline --locked` | exit 0 | `sdk-build.log` |
| package + check + fits | `cargo horizon-mod build` → `check` → `check --uses` → `fits` | `minecolonies 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…)`, **53,767 B**, sha256 **`0d014f3f5973b23c6544e4d860e10b59fd02deb3e3d72c63de377102272bbda5`**, `fits`: `0b14f8a made for` | `sdk-check.log` |
| content | `python3 tools/assemble_content.py --check`, `python3 tools/gen_assets.py --check` | `assets/minecolonies/: 15 files, byte-identical to the palette's output`, exit 0 | — |
| package contents | unzip listing | 38 entries; `LICENSE` (GPL-3.0 text) and `LICENSE-NOTICE.md` inside | — |
| reproducibility | `cargo horizon-mod build` twice | same 53,767 B and same sha256 | — |
| runtime byte-identity | `cmp src/runtime.rs <(unzip -p legacy/minecolonies-0.1.1.hnmod mixins/minecolonies/runtime.rs)` | identical, sha256 `cca59b398d2ed9c870e5820fa1ca764f0adb5d5ddf3a3783cf369dc30351299e` | — |

**Not run: `tools/e2e.sh`.** It needs a game binary built at `0b14f8a`, which does not exist on
this machine; the script was migrated to the flat layout (`--dev-mod "$PORT"`, `CONTROL=1` = the
same game without it) and refuses to start without `GAME`. The in-game claims above are therefore
the 0.1.1 behaviour that the same `runtime.rs` and the same scenario asserted — not a 0.2.0
transcript. `python3 ports/tools/check-e2e-logs.py minecolonies --strict` will not find a 0.2.0
log; the 0.1.1 logs (`e2e-dev.log`, `e2e-embedded.log`, `e2e-nightly.log`, `e2e-selftest.log`) are
kept unchanged beside them.

History — the 0.1.1 evidence, still valid for the behaviour the logic encodes:

- **Runtime tests** 36/36, written first (the red run was 0 passed / 36 failed against
  `unimplemented!()` bodies).
- **Ladder at `9f42064`**: fmt, clippy (0 warnings), `stage|check|test|build` green, embedded tests
  `game::minecolonies::tests` 36 passed (`nightly-*.log`).
- **e2e at `9f42064`** (`EMBEDDED=1`, `e2e-nightly.log`): `PASS` + `EXIT:0` — the colony founds at
  64 blocks, both refusals hold (including 120 across / 120 up), the citizen joins, takes the
  farmer's job, harvests and walks, a citizen with no colony does nothing, and the colony survives
  quit + relaunch (harvests 2 → 3); `EMBEDDED=1` control knows no mod id; `SELFTEST=1` exits 1.
- **Package 0.1.1**: `.hnmod` 38,828 B sha256 `c7bfea6a…` (archived; the older `dc665016…` in the
  previous revision of this file was a 38,827 B build superseded by the bytes on disk).

## Changelog for the resubmission (verbatim)

Supersedes #951, closed because Horizon retired `.hnmod` (#17992). Changes: flat SDK package with
`api = "0.2"` and `[native]` pinned to `0b14f8a`; hooks rewritten as `impl mod_api::Hooks` methods
against typed hosts/events (6 methods mapped — `register`, `entity_place`, `level_tick_post`,
`register_commands`, `run_mod_command` and the citizen's `EntityPart::general_entity_tick` — with
2 helpers retired without a gap and 4 gaps recorded, the unverified-in-game persistence among
them); `runtime.rs` and its tests unchanged.
