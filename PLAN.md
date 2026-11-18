# MineColonies slice 1 — implementation plan

> **History.** This is the 0.1.x plan (`.hnmod` layout: `mixins/`, one helper per hook). The port
> is now a flat SDK package; the current shape, the helper → hook table and the new gaps are in
> `notes.md` §"Migration to the SDK format". The *behaviour* below is unchanged and is still the
> slice's contract.

**Goal:** a town hall founds a colony (name, radius, owner, persisted), one citizen joins it and
works one job that ticks in the world, and `/minecolonies colony|citizen|job` is the
configuration surface.

**Approach from the tier:** `deep` — a block entity (the colony record) and an entity (the
citizen) with its tick. Shape taken from `docs/modpacks.md` §"Tiers": block entity + entity tick
+ commands.

## Files

| File | Responsibility |
|---|---|
| `mixins/minecolonies/runtime.rs` | pure rules: founding refusals, distance, radius, colony record round-trip, citizen/job model, farmer step. Unit-tested, no game types. |
| `mixins/minecolonies/support.rs` | glue: reads/writes the town hall block entity, summons citizens, runs the job tick, the `/minecolonies` commands. |
| `mixins/minecolonies/use_item_on.rs` | helper: refuse an illegal town hall placement before the block is set. |
| `mixins/minecolonies/place_block.rs` | helper: found the colony on the placed town hall. |
| `mixins/minecolonies/block_entity.rs` | helper: give the town hall its block entity. |
| `mixins/minecolonies/entity_tick_listened.rs` | helper: ask for entity ticks (API 146). |
| `mixins/minecolonies/entity_tick_post.rs` | helper: the citizen's job tick. |
| `mixins/minecolonies/register_commands.rs` | helper: declare `/minecolonies …`. |
| `mixins/minecolonies/perform_command.rs` | helper: run `/minecolonies …` for the acting player. |
| `mixins/minecolonies/embedded_content.rs` | helper: write the embedded `.hmod` out (AE2 pattern). |
| `mixins/minecolonies/data/minecolonies-content.hmod` | the built content, carried inside the bundle. |
| `content/` | generated assets + declarations: town hall block/item, farmer's hut block/item, scythe item, citizen entity type, recipes, loot. |
| `tools/gen_assets.py` | the only producer of `content/assets/**` textures and models; deterministic. |
| `tools/assemble_content.py` | writes the declarations + data (models/textures call the generator). |
| `tools/e2e.sh` | the fail-capable in-game scenario. |

## Decisions (each grounded, see `FINDINGS.md`)

1. **Founding** — placing `minecolonies:blockhuttownhall` founds the colony. Legality is decided
   in `use_item_on` (the only hook that can stop a placement), so an illegal hall never appears;
   the colony is written into the block entity in `place_block`.
2. **Name** — the default is `<owner>'s Colony`; an empty name or one over **25** characters is
   refused (the mod's own rename field, `windowtownhallnameentry.xml`, caps at 25 and refuses an
   empty name, `WindowTownHallNameEntry.java:47`).
3. **Distance** — **128 blocks** horizontal (`max(minColonyDistance=8, initialColonySize=4) << 4`,
   `ColonyManager.java:313`), refused when `< 128` (`:317`). The upstream formula is *3D*
   (`BlockPosUtil.getDistance`, `BlockPosUtil.java:453`); the port uses the horizontal distance
   and records that as a fixed defect.
4. **One colony per owner** — refused after the distance check, as upstream orders it
   (`CreateColonyMessage.java:165` then `:172`).
5. **Radius** — `size_chunks * 16` blocks, square (the chunk claim is a square chunk range,
   `ChunkDataHelper.java:236-248`); starts at `initialColonySize = 4` chunks = **64 blocks**,
   capped at `maxColonySize = 20` (chunks).
6. **Citizen** — a content living type `minecolonies:citizen` copying `minecraft:villager`
   (health 20, `CitizenConstants.BASE_MAX_HEALTH`), spawned with `NoAI` and moved by the job, so
   the scenario is deterministic. It carries `minecolonies.colony=<id>` and
   `minecolonies.job=<job>` as entity tags (they persist with the entity, API 386).
7. **Job** — `farmer`, the mod's `JobFarmer`/`EntityAIWorkFarmer` net behaviour: find a mature
   crop, walk to it, then harvest it (replanted at age 0 here; upstream breaks it and replants
   from its inventory) and count one crop harvested. One work step every **40 ticks**
   (`EntityAIWorkFarmer.DEFAULT_DELAY = 40`, line 85). The farmer works only inside the colony's
   radius and only when the colony has a farmer's hut (`minecolonies:blockhutfarmer`), which is
   upstream's rule that a job needs its building. A citizen whose colony tag does not resolve
   does nothing.
8. **Commands** — `/minecolonies colony info|rename|radius`, `/minecolonies citizen hire|list`
   (`citizens` accepted as the mod's own spelling), `/minecolonies job assign|info`. They are the
   configuration surface and the harness; upstream's own command tree (`EntryPoint.java:86-132`)
   has `colony`/`citizens` but no `job`.
9. **Persistence** — the colony record lives in the town hall block entity's saved compound
   (`Colony`), which the chunk saves; the roster is `Colony/Citizens`. The pure runtime owns the
   record/field mapping so it is round-trip tested without the game.

## Tasks

- [x] 1. `FINDINGS.md` from the repo and the jar.
- [x] 2. `runtime.rs` tests first (red), then the rules; every constant with file:line.
- [x] 3. `tools/gen_assets.py` + `tools/assemble_content.py`; `--check` regenerates byte-identically.
- [x] 4. `support.rs` + one helper per injection; the plan names stamped hooks only.
- [x] 5. Ladder: fmt, clippy, test, `horizon.py stage|check|test|build`, content build, `check`.
- [x] 6. `tools/e2e.sh`: founding, refusal, job ticks, no-colony control, quit+relaunch;
      `SELFTEST=1` fails; `EMBEDDED=1` with no `--dev-mod` and its `.horizon-cache` control.
- [x] 7. Gates + `notes.md` + `VALIDATION.md`.

## Gaps (stated, not hidden)

- The other ~40 jobs (builder, miner, guard, crafter…), the builder and its schematics, the
  research tree, the colony/citizen UI, happiness, saturation, requests, raids, permissions,
  colony size growth from buildings, world-gen colonies, and the colony registry across unloaded
  chunks (the port finds colonies through loaded town hall block entities).
- The citizen is drawn with its template's villager model; `textures/entity/citizen.png` is
  generated but not yet bound to a renderer (a native renderer slice).
