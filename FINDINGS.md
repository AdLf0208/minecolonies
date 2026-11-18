# FINDINGS — MineColonies 1.1.1411 slice 1 (town hall → colony → citizen → one job)

Ported from **ldtteam/minecolonies**, branch `version/main`, commit
`3438c06fc92947b28bbd2fda29f00e207b967a22` (2026-11-11), shallow clone in
`ports/_shared/upstream/minecolonies` (analysis scratch, deleted after this file was written).
Jar: `ports/_shared/licence-cache/minecolonies-1.1.1411-1.21.1-snapshot.jar`,
sha256 `cf396583d91d2013e1c364f0b50a2670a4d5459bb3b01bfe78a5933b28f84516`.
The clone is the mod's newest main, not the exact jar; the constants below are checked against
the jar's own data files and the catalogue values quoted, and none of the read files changed
between the jar's release and this commit for the areas used (founding, jobs, config defaults).

## Licence (both halves, read from the artefacts)

- `META-INF/neoforge.mods.toml:7` — `license="GPL3"`; the repo's `LICENSE` is the GNU GPL v3
  text (35,121 B). The **code** is GPL-3.0 → portable, publishable under GPL-3.0 with its source.
- `assets/minecolonies/LICENSE` in the jar is **19 bytes, `All Rights Reserved`**. The
  **assets are ARR** → not one file of `assets/minecolonies/**` is copied. The port's textures
  and models come from `ports/minecolonies/tools/gen_assets.py` (see `notes.md` for the palette).

## The town hall and founding

| Fact | Where |
|---|---|
| Block id `minecolonies:blockhuttownhall`, 4 horizontal `facing` states | `data/minecolonies/` blockstate in the jar: `assets/minecolonies/blockstates/blockhuttownhall.json` |
| Hut blocks: hardness **10**, blast resistance **infinite**, wood sound, no occlusion | `api/blocks/AbstractColonyBlock.java:70,75,99` |
| A colony is created for the placing player with a name and a structure pack, then its owner is set and its chunks claimed | `core/colony/ColonyManager.java:92-113` |
| Founding is refused when the site is **too close to another colony** — `blockRange = max(minColonyDistance, initialColonySize) << 4`, refused when the distance is `< blockRange` | `core/colony/ColonyManager.java:311-325` |
| `minColonyDistance` default **8** chunks, `initialColonySize` default **4**, `maxColonySize` default **20** | `api/configuration/ServerConfiguration.java:164-166` |
| … so the legal radius is `max(8,4)<<4` = **128 blocks** | `ColonyManager.java:313` |
| The distance is **3D** (`sqrt(dx²+dy²+dz²)`) | `api/util/BlockPosUtil.java:453-460` |
| … while every other colony measure is 2D (`getDistanceSquared2D` `:469`, `chunkDistanceSquared` `:1237`) | same file |
| After the distance, a player who already owns a colony is refused | `core/network/messages/server/CreateColonyMessage.java:165` (distance) then `:172` (owner) |
| A colony's size is a **square chunk range** around its centre: claim `chunkX ± range`, `chunkZ ± range`, capped by `maxColonySize` | `core/util/ChunkDataHelper.java:236-248`, `:293-299` |
| Rename field: **maxlength 25**, an empty name is not applied | `resources/assets/minecolonies/gui/townhall/windowtownhallnameentry.xml`; `core/client/gui/townhall/WindowTownHallNameEntry.java:45-50` |
| Default colony name pattern `"%s's Colony"` | `resources/assets/minecolonies/lang/manual_en_us.json:547` |
| Command root `minecolonies` (alias `mc`) with subtrees `colony` and `citizens` | `core/commands/EntryPoint.java:86-132` |

## The citizen

| Fact | Where |
|---|---|
| A new colony gets `initialCitizenAmount` = **4** citizens | `api/configuration/ServerConfiguration.java:125`, `core/colony/managers/CitizenManager.java:594` |
| Citizen base max health **20** | `api/util/constant/CitizenConstants.java:84` |
| Citizen data holds colony, id, name, job, home building, saturation; full saturation 60 | `api/colony/ICitizenData.java:30`, `core/colony/CitizenData.java` |
| Citizen names are generated per colony from a name file; gender and texture chosen at random | `core/colony/CitizenData.java:515-527` |
| The citizen entity carries its colony id and citizen id and a custom name | `core/colony/CitizenData.java:552-560` |

## The job that is ported: farmer

| Fact | Where |
|---|---|
| `JobFarmer` is a job over `EntityAIWorkFarmer`; saturation factor 1.2 | `core/colony/jobs/JobFarmer.java` |
| Work delay = `max(1, 40 - skill/2)` ticks, so **40 ticks** at skill 0 | `core/entity/ai/workers/production/agriculture/EntityAIWorkFarmer.java:85,692` |
| The farmer finds a mature crop on its field, harvests it (breaks the block, drops go to the building), then replants from its inventory | `EntityAIWorkFarmer.java:661-676`, `:777-810`, `:743-769` |
| A crop is mature when `CropBlock.isMaxAge(state)` | `EntityAIWorkFarmer.java:797` |
| The job needs its building (`BuildingFarmer`); the colony's jobs are the huts it has | `core/colony/jobs/AbstractJob.java`, `api/colony/buildings/ModBuildings.java` |

## Differences the rewrite makes (deviations, not defects)

- **One citizen**, not `initialCitizenAmount` (4): the slice proves one recruit deterministically;
  the config default is recorded above.
- The farmer **replants at age 0** and counts the crop, instead of breaking it and replanting from
  an inventory: a dropped item entity would be the only observable difference, and the slice has
  no citizen inventory.
- The citizen is `NoAI` and is moved by the job, so the scenario is deterministic; upstream's
  pathfinding is not ported.
- Colonies are found through **loaded town hall block entities**; upstream keeps a per-world
  colony list that also covers unloaded chunks (gap).

## Upstream defect fixed

`ColonyManager.isFarEnoughFromColonies` measures the distance to the closest colony with
`BlockPosUtil.getDistance`, which is the **3D** distance (`BlockPosUtil.java:453-460`), while the
colony is a 2D claim (`getDistanceSquared2D` at `:469`; the chunk claim at
`ChunkDataHelper.java:236-248` is a square in x/z only). A town hall 100 blocks below another
colony therefore reads as "far enough" while the two colonies overlap on the map. The port uses
the horizontal distance with the same 128-block threshold.
