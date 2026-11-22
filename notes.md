# MineColonies — slices 1–5: town hall → colony → citizens → farmer, lumberjack, miner, the two
# jobs' buildings, the tool request and the food request

**Class:** `content` · **Tier:** `deep` · **Role:** core · **Priority:** P0 · in ATM10.
Ported from **MineColonies** `1.1.1411-1.21.1-snapshot`, jar
`ports/_shared/licence-cache/minecolonies-1.1.1411-1.21.1-snapshot.jar`
(sha256 `cf396583d91d2013e1c364f0b50a2670a4d5459bb3b01bfe78a5933b28f84516`),
upstream `ldtteam/minecolonies` branch `version/main`, commit
`3438c06fc92947b28bbd2fda29f00e207b967a22`.

**Format (2026-11-18):** the flat **SDK package** (mod API `0.2`), version **0.2.7** — one
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
  re-renders all **25** asset files **byte-identically**, and
  `python3 ports/tools/check-provenance`-style measurement says **0 jar-derived bytes**: the
  sha256 of every one of the 25 files under `assets/` is compared against the sha256 of all 22,747
  entries of the jar (`ports/_shared/evidence/minecolonies/sdk-provenance.log`; the command is in
  §"Verification").

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

### Slice 3 — the two jobs' buildings: `blockhutlumberjack` and `blockhutminer` (0.2.4 → 0.2.5)

The gap line this slice closes (`## Gaps`, as it stood before it):

> … the **buildings the new jobs would need** (a lumberjack's hut, a mine): the jobs themselves are
> ported and asserted, the two blocks are not (see the deviations).

Chosen because it is the deviation 0.2.4 itself named, it is **content + one rule** rather than a
new mechanic, and it is provable in the harness through the command that already reports a
building: `job info`. It fits `deep` without growing the port's shape — two more blocks, two more
declared block-entity types, and one more `Hooks` method (`player_destroyed_block`).

Required behaviour, from upstream's own data and its job classes (implemented, not copied):

| Rule | Source |
|---|---|
| The two blocks are `minecolonies:blockhutlumberjack` and `minecolonies:blockhutminer`; upstream's lang names them *Forester's Hut* and *Mine* | `assets/minecolonies/lang/en_us.json`: `block.minecolonies.blockhutlumberjack`, `block.minecolonies.blockhutminer` (the port's `lang/en_us.json` uses the same two names) |
| Every job has a building (`JobLumberjack`/`BuildingLumberjack`, `JobMiner`/`BuildingMiner`, `JobFarmer`/`BuildingFarmer`) and a job without its building does nothing | `AbstractJob`/`AbstractBuilding`; the same rule the farmer already had |
| Both hut recipes are planks around the scepter with the trade's tool in the middle: `XTX/XBX/XXX` (`T` = `structurize:sceptergold`, `B` = `wooden_axe` / `wooden_pickaxe`) | `data/minecolonies/recipe/blockhutlumberjack.json`, `blockhutminer.json` |
| A hut block is a colony block: hardness 10, blast resistance infinite, wood sound, no occlusion | `AbstractColonyBlock.java:70,75,99` (the same declaration the town hall and the farmer's hut already carry) |

**TDD, in the order the ruling asks for.** The rules live in `runtime.rs` (pure), so the failing
tests came first — the two new rules as `panic!()` bodies plus the generalised `building()`:

```
$ cargo test --offline
---- runtime::tests::every_working_job_names_the_building_its_rule_needs stdout ----
thread 'runtime::tests::every_working_job_names_the_building_its_rule_needs' panicked at src/runtime.rs:309:9:
a job's building
---- runtime::tests::job_info_names_each_building_the_way_the_command_says_it stdout ----
thread 'runtime::tests::job_info_names_each_building_the_way_the_command_says_it' panicked at src/runtime.rs:340:5:
not implemented: a job's building status
...
test result: FAILED. 54 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
```

(`ports/_shared/evidence/minecolonies/sdk-test-slice3-red.log`; the four are the two new rules,
`the_new_jobs_round_trip_through_their_ids` (the old expectation was `Lumberjack.building() == None`)
and `the_huts_are_the_three_blocks_the_port_carries`.) Then the rules, the glue and the content;
`sdk-test.log` is the green run: **59 tests**.

**What changed in the rules**: `Job::building()` answers each working job's own block
(`LUMBERJACK_HUT`, `MINER_HUT` join `FARMER_HUT`), `Job::building_label()` names it the way the
command says it ("lumberjack's hut", "mine"), `building_status(job, present)` produces
`"lumberjack's hut present"` / `"… missing"` / `"no building"` for the unemployed, and `is_hut(id)`
reads the one `HUTS` list the registration, the marker and the break handler share.

**What changed in the glue**: the hut marker became **one** attachment type (`minecolonies:hut`)
whose value is *which building* it is, so `has_building(colony, job.building())` is one query for
every job rather than a farmer special case; `register` declares three block-entity types from
`HUTS`; the placement queue's "what does this placement become" became an enum
(`Founding`/`Building`) rather than an `owner: Option<…>`; and **`Hooks::player_destroyed_block`**
removes a broken hut's marker, so the building is gone when the block is.

**The harness's assertions** (`tools/e2e.sh` L1–L3 and M1, at `0a875da`):

```
== L1: the lumberjack's hut is its building: without it the assignment is refused ==
  without its hut: A lumberjack needs lumberjack's hut in the colony: place minecolonies:blockhutlumberjack.
== L2: the hut is placed, job info reads it, and the tree falls ==
  Bo: job Lumberjack, colony Riverside (3 crops harvested), 0 logs felled, 0 blocks mined, lumberjack's hut present.
  after the fell: soil at 2004,101,-1 is 'minecraft:dirt', stump is 'minecraft:oak_sapling',
    chest [{"count":4,"item":"minecraft:oak_log"}]
== L3: breaking the hut takes the building with it ==
  the break dropped {"minecolonies:blockhutlumberjack":1}
  after the break: Bo: job Lumberjack, …, lumberjack's hut missing.
== M1: the mine is the miner's building ==
  Cass: job Miner, colony Riverside (…), mine present.
```

and the brief's own case: with the hut absent a citizen assigned `lumberjack` beside a whole
four-log tree leaves it **whole for 200 ticks** (L1's counterfactual, after the refusal), and with
the hut placed the same tree reads 4 × air with 4 `oak_log` in the chest (L2). The break is made
**in survival with an iron axe** (`gamemode survival` for the one action): a creative break drops
nothing in vanilla, so a creative test could never run the hut's own loot table — this one asserts
the drop, `{"minecolonies:blockhutlumberjack":1}`, and then goes back to creative. The break is
fail-capable on its own: `HUT_SELFTEST=1` inverts L3 (a broken hut must still read "present") and
fails `the broken hut reads missing, so the marker went with the block` — a port whose marker
outlived its block passes that leg and fails the normal one
(`../_shared/evidence/minecolonies-e2e/e2e-hut-selftest.log`).

Both new recipes are crafted in a real table with their ingredients consumed
(`tools/craft_check.py`, phase N):

```
  lumberjack's hut: planks around a crafting table and a wooden axe ->
    {"count": 1, "item": "minecolonies:blockhutlumberjack", "slot": 200}
  mine: planks around a crafting table and a wooden pickaxe ->
    {"count": 1, "item": "minecolonies:blockhutminer", "slot": 200}
  lumberjack's hut with a stick where the axe goes -> null
```

### Slice 4 — the tool request: every job's trade tool, taken from the colony's chest (0.2.5 → 0.2.6)

The gap line this slice closes (`## Gaps`, as it stood before it):

> The other ~35 jobs (builder, guard, crafter, cook, …), … **the request system**, raids,
> permissions and ranks …

and, from the deviations, *"the scythe is a plain item with no harvest behaviour of its own (the
job does the harvesting)"*. The first family of upstream's request system is the **tool request**:
a job declares the tool it needs, a worker without it does not work, the colony records the
request, and a tool the colony holds is taken into the citizen's one held slot. It gives the
carried `minecolonies:scythe` its first behaviour (the farmer's trade tool). It fits `deep`
without growing the shape — one more rule in `runtime`, one equipment read/write and one deferred
pickup in the glue, one optional record field and one command.

Required behaviour, from upstream's own data and its request classes (implemented, not copied):

| Rule | Source |
|---|---|
| The tool classes are `axe`, `pickaxe` and `hoe`; a request names a class **and a minimum level** (`Tool of class:` / `with minimal level:`), and the levels are `0` Wood/Gold … `2` Iron | `assets/minecolonies/lang/en_us.json`: `com.minecolonies.coremod.tooltype.*`, `com.minecolonies.coremod.request.tool.pretype` / `.preminlevel`, `com.minecolonies.coremod.toollevel.2` |
| A failed tool need is a **`ToolRequest`** in the colony's request system, not a silent stall | `com/minecolonies/core/colony/requestsystem/requests/StandardRequests$ToolRequest.class` |
| A citizen's tools live in its **equipment**, and the hand is the slot a job reads | vanilla `LivingEntity.getItemBySlot(EquipmentSlot.MAINHAND)`; the pin's `LivingRef::hand_item` / `set_hand_item` (`living.rs:195-199`, `:533`) |
| A worker's tool arrives from the colony's store, not from the worker's own pocket | the port's 0.2.4 rule (the colony's chest is the citizen's store); a citizen has no inventory |
| A job's building and its tool are both prerequisites of the job's rule | `AbstractJob`/`AbstractBuilding` (the building) and `ToolRequest` (the tool) |

The job's tool in this port, one concrete item per job (upstream asks for a class and a level; a
colony at one tier names the item the readout prints):

| Job | Tool | Why |
|---|---|---|
| `farmer` | `minecolonies:scythe` | the tool the port carries for the farmer; the id the readout prints |
| `lumberjack` | `minecraft:iron_axe` | the axe class (`tooltype.axe`), iron level (`toollevel.2`) |
| `miner` | `minecraft:iron_pickaxe` | the pickaxe class (`tooltype.pickaxe`), the same level |

**TDD, in the order the ruling asks for.** The rules live in `runtime.rs` (pure), so the failing
tests came first — nine of them, with the new rules as `panic!()`/`unimplemented!()` bodies and the
record field not yet written:

```
$ cargo test --offline
---- runtime::tests::every_working_job_names_the_tool_its_rule_needs stdout ----
thread 'runtime::tests::every_working_job_names_the_tool_its_rule_needs' panicked at src/runtime.rs:358:9:
a job's tool
---- runtime::tests::a_job_without_its_tool_does_not_work stdout ----
thread 'runtime::tests::a_job_without_its_tool_does_not_work' panicked at src/runtime.rs:412:5:
not implemented: a job's tool
---- runtime::tests::job_info_reports_the_hand stdout ----
thread 'runtime::tests::job_info_reports_the_hand' panicked at src/runtime.rs:420:5:
not implemented: the citizen's hand
---- runtime::tests::the_request_line_reads_the_way_the_command_says_it stdout ----
thread 'runtime::tests::the_request_line_reads_the_way_the_command_says_it' panicked at src/runtime.rs:427:5:
not implemented: the colony's requests
...
test result: FAILED. 59 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
```

(`ports/_shared/evidence/minecolonies/sdk-test-slice4-red.log`; the nine are the five new rules,
the request list's three, the record's new field and the support round trip.) Then the rules, the
glue and the in-game assertions; `sdk-test.log` is the green run: **68 tests**.

**What changed in the rules** (`src/runtime.rs`):

- `Tool { job, item }`, `LUMBERJACK_TOOL`/`MINER_TOOL`/`FARMER_TOOL`, `Job::tool()`;
- `tool_missing(job, held)` — the job's rule does not run without its item;
- `hand_status(job, held)` — `holding minecraft:iron_axe` / `holding nothing` /
  `holding X, not Y` / `no tool`;
- `Request { citizen, name, job, item }`, `Colony::{request, settle, open_request}`
  (`request` answers whether the record changed, so an unchanged request is not rewritten every
  40 ticks), and `requests_line` / `requests_status` for the two readouts;
- the record's `Requests` field, **optional on read** (a 0.2.5 colony loads with none) and written
  as a list of `{Citizen, Name, Job, Item}` compounds.

**What changed in the glue** (`src/support.rs`):

- `step_job` reads the citizen's main hand (`LivingRef::hand_item`), and a job whose tool is not
  there records the request and **returns before the walk/harvest/fell/dig**;
- the pickup is a **queue applied at `level_tick_post`** (`Pickup`), because a block-entity write
  made inside a tick is overwritten by that entity's own write-back — the same rule the deliveries
  follow. The tool is extracted from the colony's chest and written with
  `LivingRef::set_hand_item(host, Hand::Main, …)`; the request is settled in the same flush. A hand
  write that fails puts the tool back in the store (and on the ground if the store cannot take
  it), so a failed write never loses the item;
- `job info` appends the hand, `colony info` appends `N requests open`, `citizen list` appends
  `, needs <item>` for a citizen with an open request, and `/minecolonies requests` reconciles the
  record against the live hands before it prints (so a row whose tool arrived cannot survive a
  read). `job assign` drops the previous job's request;
- `read_fields` now lists **`Felled`**, **`Dug`** and `Requests`: see the port defect below.

**The harness's assertions** (`tools/e2e.sh` H, I, K, L1–L3, M1, at `0a875da`):

```
== H: the job's tool is a request, then it is taken from the colony's chest ==
  Ada: job Unemployed, …, no building, no tool.
  Ada: job Farmer, …, farmer's hut present, holding nothing.
  1 request: Ada needs minecolonies:scythe
  the unarmed farmer left the mature crop at age 7 and the request open
  Ada: job Farmer, …, farmer's hut present, holding minecolonies:scythe.
  the scythe left the chest for Ada's hand and the request cleared
== K: the colony survives quit and relaunch ==
  Ada: job Farmer, …, farmer's hut present, holding minecolonies:scythe.      (after the relaunch)
== L2: the hut is placed, the tool is requested, then taken, and the tree falls ==
  Bo: job Lumberjack, …, lumberjack's hut present, holding nothing.
  1 request: Bo needs minecraft:iron_axe
  without an axe the tree is whole and Bo's request is open
  Bo: job Lumberjack, …, lumberjack's hut present, holding minecraft:iron_axe.
  the axe left the chest for Bo's hand and the request cleared
  after the fell: soil at 2004,101,-1 is 'minecraft:dirt', stump is 'minecraft:oak_sapling',
    chest [{"count":4,"item":"minecraft:oak_log"}]
== M1: the mine is the miner's building, and the pickaxe is its tool ==
  Cass: job Miner, …, mine present, holding nothing.
  1 request: Cass needs minecraft:iron_pickaxe
  without a pickaxe the column is whole and Cass's request is open
  Cass: job Miner, …, mine present, holding minecraft:iron_pickaxe.
  the pickaxe left the chest for Cass's hand and the request cleared
```

Every tool leg has its counterfactual **before** the tool arrives (the mature crop stays at age 7,
the four-log tree stays whole, the eight-block column stays whole), and the "held" read is
inverted by `TOOL_SELFTEST=1` — the run then fails at
`selftest: Ada holds minecolonies:scythe, so the hand was written`, so a port whose equipment write
silently did nothing passes the inverted leg and fails the normal one. `SELFTEST=1` inverts both
that leg and the miner's depth; `MINER_SELFTEST=1` inverts the depth alone.

**Port defect fixed in this slice.** The saved colony record was **losing its `Felled` and `Dug`
counters on every read**: `colony_fields` wrote both (0.2.4), the codec's `read_fields` did not
list their keys, and `colony_from_fields` reads an absent counter as zero — so a colony that had
felled 8 logs read 0 after a relaunch. It was invisible in 0.2.5 because `bump_colony` reads the
live attachment (not the saved compound), so only the load path was affected. The support test
`a_colony_round_trips_through_its_saved_compound` (encode → decode) is the failing test that
pins the whole record, `Felled`/`Dug`/`Requests` included; it failed before the `read_fields`
repair. Not an upstream defect — this port's own decoder — so there is nothing to report upstream.

### Slice 5 — the food request: a hungry citizen is fed from the colony (0.2.6 → 0.2.7)

The gap line this slice closes (`## Gaps`, as it stood before it):

> The other ~35 jobs (builder, guard, crafter, cook, …), … happiness, saturation, **the rest of the
> request system** (deliveries, crafting, food, pickups and the request GUI — the **tool** family is
> ported in 0.2.6), raids, permissions and ranks …

The **food** family is the request system's next slice, and the 0.2.6 report's next-slice proposal:
the citizen gains a saved **saturation**, every work tick spends one point of it, zero stops the
work and opens a **food request**, and one bread from the colony's chest fills the citizen back up,
clears the request and lets the job run. It fits `deep` without growing the shape: one more rule
set in `runtime`, one more saved field in the roster row, one more request kind, one more branch of
the pickup queue the tool family already uses.

Required behaviour, from upstream's own classes (implemented, not copied):

| Rule | Source |
|---|---|
| A citizen has a **saturation** saved with it, full at **60**; a new citizen starts full | `api/colony/ICitizenData.java:30` (`MAX_SATURATION = 60`); `core/colony/CitizenData.java:526`, `:1337`, `:1460` (`TAG_SATURATION`) |
| Saturation is spent by living and working: an idle drain on a timer plus an extra cost per action | `core/colony/CitizenData.java:1164-1171`; `api/entity/citizen/AbstractEntityCitizen.java:576-584`; `api/util/constant/CitizenConstants.java:61` |
| A citizen at **saturation 0** is slowed and does not work; the colony must feed it | `core/entity/citizen/EntityCitizen.java:812-825`; `core/entity/ai/workers/CitizenAI.java:286-299` (`shouldEat`) |
| Food is a **`FoodRequest`** in the colony's request system, not a silent stall | `core/colony/requestsystem/requests/StandardRequests$FoodRequest.class`; `com.minecolonies.coremod.request.food` = *Menu* |
| A meal restores saturation towards full | `core/entity/ai/minimal/EntityAIEatTask.java:227,378`; `api/colony/requestsystem/requestable/Food.java` (a count and a minimum nutrition) |
| The job's own saturation factor scales its cost (the farmer's is 1.2) | `core/colony/jobs/JobFarmer.java:42-45`; `api/colony/jobs/IJob.java:320-326` |

**TDD, in the order the ruling asks for.** The rules live in `runtime.rs` (pure), so the failing
tests came first — the new rules as `panic!()`/`unimplemented!()` bodies and the two saved-field
plumbing left unread:

```
$ cargo test --offline
thread 'runtime::tests::every_working_job_names_the_food_its_citizen_needs' panicked at src/runtime.rs:395:9:
a job's food
thread 'runtime::tests::a_work_tick_spends_one_point_of_the_citizens_saturation' panicked at src/runtime.rs:695:9:
not implemented: a work tick's saturation
thread 'runtime::tests::a_hungry_citizen_asks_the_colony_for_food' panicked at src/runtime.rs:695:9:
thread 'runtime::tests::zero_saturation_is_hunger' panicked at src/runtime.rs:497:5:
not implemented: a hungry citizen
thread 'runtime::tests::job_info_reads_the_saturation_the_way_the_command_says_it' panicked at src/runtime.rs:504:5:
not implemented: the citizen's saturation
thread 'runtime::tests::the_record_carries_the_request_kind' panicked at src/runtime.rs:2357:9:
assertion `left == right` failed: left: None, right: Some(Tool)
thread 'support::tests::a_colony_round_trips_through_its_saved_compound' panicked at src/support.rs:1780:9:
left: … saturation: 60 }, … need: Tool … (the saved rows read their defaults)
test result: FAILED. 71 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
```

(`ports/_shared/evidence/minecolonies/sdk-test-slice5-red.log`; the nine are the food sibling, the
spend, hunger, the request, the meal, the saturation readout, the record's saturation, the record's
request kind and the support round trip.) Then the rules, the glue and the in-game assertions;
`sdk-test.log` is the green run of the same suite: **80 tests**.

**What changed in the rules** (`src/runtime.rs`):

- `MAX_SATURATION = 60`, `SATURATION_STEP = 1`, `FOOD = "minecraft:bread"`, `Food { item }`;
- `Job::food()` — the sibling of `Job::tool()`: every working job eats the colony's one meal, the
  unemployed citizen eats nothing;
- `is_hungry(saturation)` (zero), `Colony::{spend_saturation, feed}`, `saturation_status`;
- `Need { Tool, Food }` on every `Request`, so one list carries both families and the readout can
  still print `1 request: Bo needs minecraft:bread`;
- the roster row's **`Saturation`**, optional on read (a 0.2.6 roster reads full), and the request
  row's **`Kind`**, optional on read (a 0.2.6 request reads as the tool family it was);
- `Value::{Citizens, Requests}` became rows (`CitizenRow`, `RequestRow`) with `Option` saved fields,
  so the optional-on-read rule is a unit-tested part of the mapping, not a decoder detail.

**What changed in the glue** (`src/support.rs`):

- `step_job` spends one point of saturation on the citizen's own work tick, after its building and
  its tool, and returns before the job when the value is zero, recording the **food** request and
  queueing the meal;
- the pickup queue took a `need`, so one queue serves both families: a tool goes to the main hand,
  a **meal is eaten** — `Colony::feed` sets the citizen back to full and the request settles in the
  same write;
- `job info` prints the saturation (`saturation 46/60`, `saturation 0/60 (hungry)`), and
  `/minecolonies requests`'s reconciliation opens the food request for a hungry citizen that holds
  its tool;
- `encode_colony`/`read_fields` write and read the two optional fields; both decoder tests fail if
  the list stops naming them (the 0.2.6 `Felled`/`Dug` defect is why they exist).

**The harness's assertions** (`tools/e2e.sh` O, at `0a875da`):

```
== O: a hungry worker asks the colony for food, and one bread feeds it ==
  Bo: job Lumberjack, …, saturation 46/60, lumberjack's hut present, holding minecraft:iron_axe.
  Bo starts at saturation 46/60
  saturation 46 -> 41 over 200 ticks, the whole tree still stands
  Bo: … saturation 0/60 (hungry), lumberjack's hut present, holding minecraft:iron_axe.
  1 request: Bo needs minecraft:bread
  at zero saturation the tree is whole and the colony owes Bo a bread
  Bo: … saturation 59/60, lumberjack's hut present, holding minecraft:iron_axe.
  the bread left the chest, Bo reads fed and the request cleared
  the fed lumberjack felled the tree at 2022,-8
```

Four assertions, each able to fail: the read falls over 200 ticks (a port that never spends
saturation reads the same value); the four-log tree is **whole** while it falls, because the tree
stands 18 blocks from Bo (outside the lumberjack's 16-block search) and only moves into reach after
the meal; at zero the readout is `saturation 0/60 (hungry)` and the colony's **one** request is
`Bo needs minecraft:bread`; the bread leaves the chest, the citizen reads fed and the tree falls.
`SATIETY_SELFTEST=1` inverts the "fed" read (the citizen must still read hungry after eating), so a
port whose meal was never applied passes the inverted leg and fails the normal one
(`../_shared/evidence/minecolonies-e2e/e2e-satiety-selftest.log`).

**Where it was read (analysis, not code).** The clone `ldtteam/minecolonies` at commit
`3438c06fc92947b28bbd2fda29f00e207b967a22` (the commit already named at the top of this file;
`git log -1` confirms it), areas `api/colony/ICitizenData.java`, `api/colony/jobs/IJob.java`,
`core/colony/CitizenData.java`, `core/colony/jobs/JobFarmer.java`,
`api/entity/citizen/AbstractEntityCitizen.java`, `core/entity/citizen/EntityCitizen.java`,
`core/entity/ai/workers/{CitizenAI,AbstractEntityAIBasic}.java`,
`core/entity/ai/minimal/EntityAIEatTask.java`,
`core/colony/requestsystem/requests/{StandardRequests,StandardRequestFactories}.java`,
`api/colony/requestsystem/requestable/Food.java` and
`api/util/constant/CitizenConstants.java`; the jar's `assets/minecolonies/lang/en_us.json` for the
request keys (`com.minecolonies.coremod.request.food`). The clone was deleted after this file was
written. The rewrite differs where `notes.md`'s deviations say so (integer steps, one meal, the
chest as the store, one request row per citizen).

**The e2e's arrangement, stated:** a citizen that holds its job drains every work tick, so by the
long felling and mining phases the *idle* farmer had also run out and the colony owed it a bread —
correct behaviour, but it makes a one-request readout ambiguous. The leg therefore **parks** the
farmer's job after K (its phases are over) and the miner's after M1, so each request readout names
one worker. That is a scenario arrangement, not a port rule; every request is a real one.

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
| A job needs its building (a farmer the farmer's hut, a lumberjack the lumberjack's hut, a miner the mine) | `JobFarmer`/`BuildingFarmer`, `JobLumberjack`/`BuildingLumberjack`, `JobMiner`/`BuildingMiner`, `ModBuildings` |
| Citizen base health 20; full saturation 60; the entity carries its colony id and a custom name | `CitizenConstants.java:84`; `ICitizenData.java:30`; `CitizenData.java:552-560` |
| One colony per player | `CreateColonyMessage.java:172-183` |
| Command roots `minecolonies` (alias `mc`), subcommands `colony` and `citizens` | `EntryPoint.java:86-132` |
| A job's tool is a request with a class **and a minimum level**; a tool need that cannot be met is a `ToolRequest` in the colony's request system | `en_us.json` `tooltype.*`/`toollevel.*`/`request.tool.pretype`; `StandardRequests$ToolRequest.class` |
| A citizen's tools are its **equipment**; the hand is the slot a job reads, and it saves with the entity | `LivingEntity.getItemBySlot(MAINHAND)`; the pin's `LivingRef::hand_item`/`set_hand_item` |
| A worker does not do its job without its trade tool (the building and the tool are both prerequisites) | `AbstractJob`/`AbstractBuilding` plus the `ToolRequest`; the port's rule |
| The colony's request list is what a readout shows, and a request that can be met is not left open | the port's `StandardRequests$ToolRequest` reading, narrowed to the tool family |
| A citizen's saturation is full at **60**, saves with the citizen, and a new citizen starts full | `ICitizenData.java:30`; `CitizenData.java:526,1337,1460` |
| Saturation is spent by work; at **0** the citizen stops and the colony owes it **food** | `CitizenData.java:1164-1171`; `CitizenAI.shouldEat`; `StandardRequests$FoodRequest` |
| A meal restores saturation; one food request is a count of food and a minimum nutrition | `EntityAIEatTask.java:227,378`; `Food.java` |

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
| `lumberjack_timber` | `#6e5433` | lumberjack's hut texture, walls |
| `lumberjack_roof` | `#3f6b2a` | lumberjack's hut texture, top three rows |
| `lumberjack_door` | `#2f2413` | lumberjack's hut texture, door |
| `miner_stone` | `#6b6f77` | mine texture, the stone face |
| `miner_timber` | `#4a3826` | mine texture, the head-frame beams |
| `miner_lamp` | `#e3c05a` | mine texture, the lamp |
| `scythe_blade` | `#c8d2da` | scythe sprite, blade |
| `scythe_handle` | `#7a522e` | scythe sprite, shaft |
| `citizen_skin` | `#d9a47a` | citizen texture, upper half |
| `citizen_shirt` | `#3f6f8f` | citizen texture, lower half |

Model templates (every model JSON is one of the three): `cube` (`minecraft:block/cube_all`) for the
town hall, `slab` (one box `[0,0,0]..[16,8,16]`) for the three huts, `cross`
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
- **A broken hut**: `Hooks::player_destroyed_block` (a player's break, after the removal) drops the
  `HUT` marker at that position, so a colony never claims a building that is not standing.
- **The held tool**: `LivingRef::hand_item(host, Hand::Main)` reads the citizen's one held slot and
  `LivingRef::set_hand_item` writes it (the handle's methods, not the host trait — the `living_*`
  methods on the host traits are the hidden backend). The read happens in the job tick; the
  **write and the chest extraction** are a `Pickup` queue applied at `level_tick_post`, because a
  block-entity write inside a tick is overwritten by that entity's own write-back.
- **The food request**: the same `Pickup` queue as a tool, with a `need`: a meal is taken from the
  colony's chest at `level_tick_post` and **eaten** (`Colony::feed` fills the citizen and settles the
  request in one write), where a tool goes into the main hand. The saturation itself lives in the
  roster row and saves with the colony's codec, so it survives a relaunch like the counters do.
- **Commands**: `Hooks::register_commands` pushes `Command::new("<usage>")` usages;
  `Hooks::run_mod_command` receives the **parsed** invocation (literals + typed arguments) instead
  of a raw line. `/minecolonies requests` reads the record's open requests after reconciling them
  with the live hands.

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

The six `Hooks` methods plus the one `EntityPart` method are the whole mod surface;
`uses.json` inside the package lists them (`hook:entity_place`, `hook:general_entity_tick`,
`hook:player_destroyed_block`, `hook:register_commands`, `hook:run_mod_command`,
`hook:level_tick_post`, `macro:horizon_part`), which
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
- ~~**New in 0.2.2 (slice 2):** the two new jobs need **no building**~~ — **closed in 0.2.5**: the
  port now carries `blockhutlumberjack` and `blockhutminer`, `Job::building()` answers each working
  job's own block, and an assignment without it is refused (upstream's own rule). The buildings'
  `blueprints/**` remain jar art this port may not carry, so the huts are **blocks without
  schematics**: the building is the rule's token, and the builder (which would raise the schematic
  from it) is still a gap.
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
- **New in 0.2.6 (slice 4):** a tool request names **one concrete item** per job, not upstream's
  "class + minimum level": the colony carries one tier (iron; the farmer's scythe), so
  `Tool { job, item }` is the whole request and the readout prints the item. A colony that carried
  several tiers would need the class/level form.
- **New in 0.2.6 (slice 4):** the **colony's chest is also the tool store**. Upstream asks the
  colony's request system, which a builder/rack/delivery chain fulfils; this port has no racks, so
  the one chest the 0.2.4 slice already made the worker's delivery point is where a requested tool
  comes from. The citizen takes it into its **main-hand slot** (`living_set_hand_item`), which is
  the one held slot this slice models; there is no second hand, backpack or tool damage.
- **New in 0.2.6 (slice 4):** the request list is **the colony record's**, reconciled against the
  live hands by `/minecolonies requests` (and refreshed by each job tick) rather than by upstream's
  standalone request manager — the record is what saves with the hall, and the readout is its
  repair point. Only the **tool** family is ported: deliveries, crafting, food, pickups and the
  request GUI stay gaps.
- **New in 0.2.6 (slice 4):** a tool is taken at `level_tick_post`, one tick after the request, not
  in the same tick: the extraction writes the colony's chest, and a block-entity write inside a
  tick is overwritten by that entity's own write-back (the delivery rule from 0.2.2). The hand write
  could happen at once, but the chest cannot.
- **New in 0.2.7 (slice 5):** the saturation rule is **one point per work tick** (`SATURATION_STEP
  = 1`) on the citizen's own 40-tick cadence, spent only where its job can run (its building stands
  and its tool is in hand). Upstream spends 0.3 per 1200 idle ticks plus an action cost, and scales
  it by job and housing level (`computeSaturationConsumptionFactor`, `JobFarmer.getSaturationFactor`
  = 1.2); this slice states the simpler rule instead of carrying the timer and the factors. A
  consequence a user notices: an employed but idle citizen becomes hungry within ~2,400 ticks.
- **New in 0.2.7 (slice 5):** a colony's request list is **one row per citizen**, so a citizen that
  is both hungry and missing its tool asks for the **tool** first (it cannot work without either
  way); the food request appears once the tool is in its hand. Upstream keeps a separate request
  per need.
- **New in 0.2.7 (slice 5):** one `minecraft:bread` is a **full meal** — the food request's item is
  one concrete food (as the tool request names one concrete tool), and eating fills the citizen to
  `MAX_SATURATION` (upstream fills by the food's nutrition from a colony menu). The chest is the
  store, as it already is for tools and deliveries.

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

**Data that could never match**: the town hall recipe names `structurize:sceptergold`, and so do
the two huts' recipes (`PCP`'s `C` slot). Structurize is MineColonies' parent mod and is not ported
here, so the recipe could never match at this pin (the same rule as `sushigocrafting`'s knife tag).
The port's recipes keep the shapes and use vanilla ingredients (planks + a crafting table; planks +
an iron hoe; planks + a crafting table + a wooden axe / wooden pickaxe; iron ingots + sticks),
which is recorded here rather than shipped broken.

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
| `minecolonies:blockhutlumberjack` | tested | L1: with none in the colony the lumberjack assignment is refused by name and the tree stays whole; L2: placed, `job info` reads "lumberjack's hut present" and the tree falls; L3: broken, the block is air and the same command reads "missing"; N: its recipe crafts it |
| `minecolonies:blockhutminer` | tested | M1: placed, `job info` reads "mine present" before the miner is assigned and works; N: its recipe crafts it |
| `minecolonies:blocks/blockhuttownhall` | tested | A: the block's own loot table (the declared shape; the block has no `requires_tool`, so a hand break drops it) |
| `minecolonies:blocks/blockhutfarmer` | tested | F: same declared shape, same tag (`minecraft:mineable/axe`, carried by `data/minecraft/tags/block/mineable/axe.json`) |
| `minecolonies:blocks/blockhutlumberjack` | tested | L3: a player's break removes it and drops it — the same declared shape and the same axe tag |
| `minecolonies:blocks/blockhutminer` | tested | M1: placed and read back; same declared shape, same axe tag |
| `minecolonies:entities/citizen` | tested | G/M: hired, counted live, and the roster is read back; the citizen entity type is what the hire spawns |
| `minecolonies:scythe` | tested | N: crafted in a real table, read back from the inventory; H: it is the **farmer's trade tool** — no scythe in the hand leaves the mature crop at age 7 and `/minecolonies requests` reads `1 request: Ada needs minecolonies:scythe`; the scythe in the chest is taken into the hand, the request clears, and the harvest runs; K: the hand survives the relaunch |
| `tool_request` | tested | H: `1 request: Ada needs minecolonies:scythe` → `requests: none`; L2: `1 request: Bo needs minecraft:iron_axe` → the axe in the hand → `requests: none`, and the tree is whole until it arrives; M1: the same for `minecraft:iron_pickaxe` and the column; each leg's unarmed counterfactual is in the same run |
| `food_request` | tested | O: `job info` reads `saturation 46/60` → `41/60` over 200 ticks with the whole tree standing; at `0/60 (hungry)` the colony's one request is `Bo needs minecraft:bread`; one bread in the chest leaves it, the citizen reads `59/60` (a work tick after the meal), the request clears and the tree falls; `SATIETY_SELFTEST=1` inverts the fed read |
| `citizen saturation` | tested | O: the read falls one point per work tick, stops the job at zero and is restored by the meal; K: the roster (saturation included) survives quit and relaunch; `support::tests::a_colony_round_trips_through_its_saved_compound` pins the encoded field and `a_saved_roster_row_without_its_saturation_reads_full` pins the 0.2.6 read |
| `block_entity` | tested | A/B/K: the town hall's block entity holds the colony attachment, which round-trips and survives a relaunch; F/H, L2, M1: each hut's marker is found by `has_building` and lost on the break (L3). All four types are declared in code (`Registrar::block_entity_type`), not in `data/`, which is why the tool sees no id for them |
| `*` | not ported | everything else the jar carries: the other ~90 blocks and ~234 items, the other ~35 jobs, the research tree (`data:researches`), crafter recipes, colony/visitor/name data, the other ~148 loot tables, the recipes the port does not carry, the item/block/entity tags, and the world-gen families (`worldgen:structure`, `worldgen:structure_set`, `worldgen:template_pool`, `worldgen:processor_list`, `tag:worldgen`). The builder's `blueprints/**` are jar **art** and may not be carried at all (`blocked(jar-assets)`). The gap list below is the prose form of this row. |

The commands (`colony info/rename/radius`, `citizen hire/list`, `job assign/info`, `requests`) are
not a family the tool inventories; every one of them is called and its **parsed** reply asserted in
the leg (A, B, G, H, I, L, M — `/minecolonies requests` in H, L2 and M1), and an unknown job is
refused by name.

## Gaps (stated, not hidden)

Content and behaviour, unchanged from 0.1.1:

- The other ~35 jobs (builder, guard, crafter, cook, …), the builder and its schematics
  (`blueprints/**`), the research tree (`data/minecolonies/researches/**`), the colony/citizen UI,
  happiness, **the rest of the request system** (deliveries, crafting, pickups and the request GUI —
  the **tool** family is ported in 0.2.6 and the **food** family in 0.2.7), raids, permissions and
  ranks, colony size growth from buildings, worldgen colonies, and the colony registry across
  unloaded chunks. Saturation is carried as one integer per work tick, not upstream's timer plus
  job/housing factors, and the colony has no dining hall or restaurant: the chest feeds the worker.
- The citizen's own model and texture (a native renderer slice). The scythe is now the farmer's
  **trade tool** (0.2.6), but it is still a plain item: it does not harvest on use, and it takes no
  damage — the job does the harvesting with it in hand.
- The colony banner, the supply camp/ship, and every other block of the mod. The **buildings the
  three jobs need** are carried (`blockhutfarmer`, `blockhutlumberjack`, `blockhutminer`) and their
  rule is asserted both ways; what is still missing is the **builder and the buildings' level /
  schematic model** — a hut here is a block and a job's token, not a multi-level building.

Introduced or left open by the migration — **closed at 0.2.4 (2026-11-21)**: the game binary at
the campaign pin now exists and the leg ran, so the attachment **codec persistence across a
relaunch** (K), the **block entity holder** for a content-declared block (A/B), the hall Part
driving the roster (G–M) and the launcher compiling `src/` from the package directory (every
launch) are all transcripts, not compile-level claims. What remains open:

- A hut placed **before** this port's attachments existed is invisible to `has_building` until
  re-placed (no migration from 0.1.1 worlds; the formats differ anyway). **0.2.5 changes the marker
  itself**: it was `minecolonies:farmer_hut` with an empty value and is now `minecolonies:hut` whose
  value names the building, so a hut placed by 0.2.4 or earlier needs re-placing once — a
  deliberate break, because "a hut" could not say *which* building it was.
- The villager-fallback citizen does not farm (above).
- **New in 0.2.6:** the request list is saved in the colony record, so a request stays as written
  until a job tick or `/minecolonies requests` reconciles it; a citizen removed from the world
  without a roster change keeps its row until the next read. The readout is the repair point.
- **New in 0.2.7:** a citizen's saturation is saved in its roster row and spent on its work ticks.
  An unemployed citizen spends nothing (it has no work tick), where upstream's idle timer would
  still drain it; the two facts are stated in the deviation list above.
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

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

At **0.2.7 / `0a875da`** (2026-11-22), the logs under `ports/_shared/evidence/minecolonies/` and
`ports/_shared/evidence/minecolonies-e2e/`. `check` and `fits` print the sha256 of the delivered
package they were run against; `sdk-build*.log` and `sdk-sha256.log` are the package's own.

| step | command | result | log |
|---|---|---|---|
| runtime tests (red) | `cargo test --offline`, the new rules as `panic!()`/`unimplemented!()` bodies | **71 passed, 9 failed** — `a job's food` / `not implemented: a work tick's saturation` / `not implemented: a hungry citizen` / `not implemented: the citizen's saturation` / the request kind / the record's saturation / the support round trip | `sdk-test-slice5-red.log` |
| runtime tests | `cargo test --offline --locked` | **80 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| format | `cargo fmt --check` | `FMT_EXIT:0` | `sdk-fmt.log` |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2, `cmp` | **`CMP:identical`**, **81,477 B**, sha256 **`63825cc5327a584ca837f0f2d0ee428cfc737d11122a993645a2e869aea0ac92`** | `sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log`, `sdk-cmp.log` |
| registry check | `cargo horizon-mod check minecolonies.hmod` (the delivered file) | `minecolonies 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`; `81,477 bytes, SHA-256 63825cc5…ea0ac92`, exit 0 — run against that sha | `sdk-check.log` |
| compile the package's source | `cargo horizon-mod check --uses minecolonies.hmod` | exit 0 (compiles for wasm as well; the unused-item notes are the native-only Parts) | `sdk-check-uses.log` |
| fits the pin | `cargo horizon-mod fits minecolonies.hmod --build 0a875da` | `0a875da  made for`, exit 0 — run against sha `63825cc5…` | `sdk-fits.log` |
| fits another build | `cargo horizon-mod fits minecolonies.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — run against sha `63825cc5…` | `sdk-fits-other.log` |
| content | `python3 tools/gen_assets.py --check`, `python3 tools/assemble_content.py --check` | `assets/minecolonies/: 25 files, byte-identical to the palette's output`; `data/` matches the assembler; `GEN_EXIT:0`, `ASSEMBLE_EXIT:0` | `sdk-assets.log` |
| **no jar-derived asset** | `python3 tools/check_provenance.py` (every file under `assets/` hashed against every jar entry) | `tree: 25 files … 14006 bytes` / `jar-derived asset bytes: 0 (files matched: 0)` — 22,747 jar files, 20,465 distinct hashes, 8,470 under `assets/minecolonies/**` | `sdk-provenance.log` |
| package contents | `unzip -l minecolonies.hmod` | **53 files** (54 entries with the header); `LICENSE` (GPL-3.0 text) and `LICENSE-NOTICE.md` inside; **no `Cargo.lock`** | `sdk-entries.txt` |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** | — |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` (the manifest's version moved to 0.2.7, so the launcher could not reuse the 0.2.6 build) | — |
| in game | `build-slot.sh run … --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0`, phases A–O | `minecolonies-e2e/e2e-launcher.log` |
| control (the unmodded Nightly) | `GAME=$CONTROL_GAME … tools/e2e.sh` | **FAIL** at `ctl place of minecolonies:blockhuttownhall … failed` / `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` | `minecolonies-e2e/e2e-launcher-control.log` |
| control inside the same run | `CONTROL=1 … tools/e2e.sh` | the control leg says *"without the package: no town hall item"*, then the modded leg is **PASS** + `EXIT:0` (0.2.6 leg; the control path is unchanged) | `minecolonies-e2e/e2e-launcher-control-inside.log` |
| fail-capability (the meal) | `SATIETY_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: Bo reads full saturation, so the meal was applied`, `EXIT:1` | `minecolonies-e2e/e2e-satiety-selftest.log` |
| fail-capability (the hand) | `TOOL_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: Ada holds minecolonies:scythe, so the hand was written`, `EXIT:1` | `minecolonies-e2e/e2e-tool-selftest.log` |
| fail-capability (the hand + depth) | `SELFTEST=1 … tools/e2e.sh` | **FAIL** at the same held read, `EXIT:1` | `minecolonies-e2e/e2e-selftest.log` |
| fail-capability (depth) | `MINER_SELFTEST=1 … tools/e2e.sh` | **FAIL** `the miner did not dig 9 blocks above the bedrock`, `EXIT:1` (0.2.6 leg; the miner's code path is unchanged) | `minecolonies-e2e/e2e-selftest-tool-and-depth.log` |
| fail-capability (the break) | `HUT_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the broken hut reads missing, so the marker went with the block`, `EXIT:1` | `minecolonies-e2e/e2e-hut-selftest.log` |
| fail-capability (founding) | `FOUNDING_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the second town hall was refused, as it should be`, `EXIT:1` | `minecolonies-e2e/e2e-founding-selftest.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=9 ok=9 missing-exit=0 bad-exit=0`, exit 0 | — |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | `families=32 … missing_families=0 … missing=0`, exit 0; `tool_request`, `food_request` and the scythe are `tested` rows | — |

**The in-game run at this pin, 2026-11-22** (`minecolonies-e2e/e2e-launcher.log`, the phases in
order): A–E founding and both refusals (including 120 across × 200 up, the fixed defect); F the
farmer's hut, two mature crops and an **empty** colony chest; G the hire; H the **tool rule**: the
unemployed citizen has `no building, no tool`, the assignment is accepted, `job info` reads
`farmer's hut present, holding nothing`, `/minecolonies requests` reads
`1 request: Ada needs minecolonies:scythe`, the mature crop beside the unarmed farmer stays at age
7 over 200 ticks, the scythe taken from the chest clears the request, `job info` reads
`holding minecolonies:scythe`, and the chest no longer holds it; I the farmer harvests and walks;
J a citizen with no colony is inert; K the colony survives quit + relaunch **and the farmer still
holds the scythe**, with no request reopened; L the two trees with the **building and tool rules**:
the assignment is refused with no lumberjack's hut (the tree stays whole), the hut is placed →
`job info` reads "lumberjack's hut present, holding nothing" → `/minecolonies requests` reads
`1 request: Bo needs minecraft:iron_axe` → the tree stays **whole over 200 ticks** → one axe in the
chest → the request reads `none`, `job info` reads `holding minecraft:iron_axe`, the axe is out of
the chest, and the tree falls (a sapling replanted only when the chest held one, the leaves
untouched, 4 then 8 logs in the chest) → the hut is **broken** → `job info` reads "lumberjack's hut
missing" (L1–L3); M the miner's 1×1 eight-block shaft to the bedrock with the **mine** and the
**pickaxe**: the unarmed miner leaves the column whole and the request open, the pickaxe taken from
the chest clears it, and the shaft is 8 deep with 8 cobblestone in the chest (M1); O the **food
rule**: the hut is placed again, the chest is emptied and a whole four-log tree stands 18 blocks
from Bo (outside its search) → `job info` reads `saturation 46/60` → `41/60` over 200 ticks with
the tree whole → at `0/60 (hungry)` `/minecolonies requests` reads
`1 request: Bo needs minecraft:bread` and the tree is still whole → one bread in the chest leaves
it, `job info` reads `59/60` (one work tick after the meal) and the request clears → the fed Bo is
moved beside the tree and fells it; N the **five** recipes crafted in a real table with their
controls, the scythe read back and an unknown id refused. Every assertion of H, L, M and O has its
counterfactual in the same run (the citizen unarmed or hungry beside the same crop/tree/column
leaves it whole); the hand inversion is the `TOOL_SELFTEST=1` leg (and `SELFTEST=1`), the meal
inversion the `SATIETY_SELFTEST=1` leg, the depth inversion the `MINER_SELFTEST=1` leg and the break
inversion the `HUT_SELFTEST=1` leg.

`tools/e2e.sh` launches with **`--offscreen`** rather than `--headless`: phase N opens a crafting
table's menu and clicks its slots, and a menu needs a screen. Its scratch home goes under
`${E2E_TMP:-/tmp/opencode}` (the launcher's env block exports `E2E_TMP`, because `/tmp` is a 7.5 GB
tmpfs). The tree/shaft/chest setup writes with `must_setblock`, which asserts every block it wrote
and retries: a dropped chat `setblock` once left a tree standing on air and failed much later
(measured in the hut-selftest leg of an earlier run), so the setup verifies its own world. Phase O's
tree stands outside the lumberjack's search so it cannot fall while the saturation is being read;
after the meal the citizen is teleported beside it (`tp @e[…,name=Bo]`), which is what makes the
last assertion about the *job* and not about walking.

**Not carried, not claimed:** the builder's schematics and the huts' levels, the rest of the
request system (deliveries, crafting, pickups, the GUI), and everything in the gap list above. The
e2e's citizen inventory is the colony's chest, stated as a deviation, not as upstream's own building
rack, and a meal is one bread from that chest.

History — the 0.2.6 and 0.2.5 evidence stays in place, superseded by this run:

- **0.2.6 / `0a875da`** (2026-11-22): 68 tests, `77,378 B` sha256 `bc41900e…f947b6`, `check`,
  `check --uses` and both `fits` exit 0, in-game `PASS` + `EXIT:0` (A–N) with the control at
  `EXIT:1`, `TOOL_SELFTEST=1`/`SELFTEST=1`/`MINER_SELFTEST=1`/`HUT_SELFTEST=1`/`FOUNDING_SELFTEST=1`
  at `EXIT:1`; the tool request and the `read_fields` repair.
- **0.2.5 / `0a875da`** (2026-11-21): 59 tests, `71,966 B` sha256 `9fbb9898…3a635`, `check`, `check
  --uses` and both `fits` exit 0, in-game `PASS` + `EXIT:0` with the control, `HUT_SELFTEST=1` and
  `FOUNDING_SELFTEST=1` at `EXIT:1`; the two huts, the building rule for every job and the
  `player_destroyed_block` break.
- **0.2.4 / `0a875da`** (2026-11-21): 55 tests, `63,940 B` sha256 `d81dcd77…ac0aa`, `check` and
  both `fits` exit 0, in-game `PASS` + `EXIT:0` with the control, `SELFTEST=1` and
  `FOUNDING_SELFTEST=1` at `EXIT:1`; the two new jobs with **no building** (the deviation 0.2.5
  closes).
- **0.2.1 / `0b14f8a`** (2026-11-19): 39 tests after the registry review's three fixes, `54,038 B`
  sha256 `5c46f90c…`, in-game `PASS` + `EXIT:0` with the control at `EXIT:1`. Its archive is
  `retired-0.2.1/`.
- **0.2.0 / `0b14f8a`** (2026-11-18): 36 tests, clippy 0, `53,767 B` sha256 `0d014f3f…`, `fits`
  `0b14f8a made for`; **no in-game run at that pin** (no game binary existed), recorded as a gap.
- **0.1.1 / `9f42064`**: the retired `.hnmod`; 36 runtime tests first (0 passed / 36 failed against
  `unimplemented!()`), the embedded-content leg with its control, and the 38,828 B bundle
  (`e2e-dev.log`, `e2e-embedded.log`, `e2e-nightly.log`, `e2e-selftest.log`).

## Changelog for the next version (verbatim)

**0.2.7** — the fifth slice, the food request:

- **A citizen has a saturation, and it falls one point per work tick.** `Citizen` gains a saved
  **`saturation`** (full **60**, a new citizen starts full; a 0.2.6 roster loads full) and
  `job info` prints it: `saturation 46/60`, `saturation 0/60 (hungry)`. A work tick spends one
  point whether or not there is work to do.
- **At zero the worker stops and the colony records a food request.** The second family of the
  request system: `/minecolonies requests` prints `1 request: Bo needs minecraft:bread` while the
  worker holds its tool and has nothing to eat; the request row gains a **`Kind`** (`tool`/`food`,
  optional on read so a 0.2.6 request still loads as a tool request).
- **One bread from the colony's chest feeds the citizen**: it is eaten at the level's end, the
  citizen reads full again, the request clears and the job runs — `Job::food()` is the sibling of
  `Job::tool()`, and the pickup queue that brings a tool now also brings a meal.
- The in-game leg gained **O**: a lumberjack below a whole tree that stands outside its search is
  read at `46/60` → `41/60`, the tree stays whole, at `0/60 (hungry)` the colony owes a bread, one
  bread in the chest leaves it, the citizen reads fed, the request clears and the fed lumberjack
  fells the tree. `SATIETY_SELFTEST=1` inverts the "fed" read, so the assertion can fail.

**0.2.6** — the fourth slice, the tool request:

- **A job needs its trade tool, and the colony records the request.** `Tool { job, item }` in the
  rules: the farmer's is `minecolonies:scythe`, the lumberjack's `minecraft:iron_axe`, the miner's
  `minecraft:iron_pickaxe`. Without it in the citizen's **main hand** the worker does not walk,
  harvest, fell or dig; the colony's record gains a **`requests`** list (optional on read, so a
  0.2.5 colony still loads) and `/minecolonies requests` prints `1 request: Bo needs
  minecraft:iron_axe` or `requests: none`.
- **The tool comes from the colony's chest**: one matching item is taken into the citizen's held
  slot at the level's end, and the request clears — the first behaviour of the carried scythe,
  which is now the farmer's tool. `job info` reports the hand (`holding minecraft:iron_axe`,
  `holding nothing`), `colony info` reports the open count and `citizen list` appends
  `, needs <item>`.
- **The saved record keeps its counters**: the codec's field list had stopped reading `Felled` and
  `Dug`, so a colony's log/block totals reset to zero across a relaunch. Fixed, and pinned by an
  encode → decode round-trip test of the whole record, `Requests` included.
- The in-game leg gained the tool legs of H, L2 and M1: each worker is left **unarmed** first
  (the mature crop stays at age 7, the four-log tree and the eight-block column stay whole) with
  its request open, then takes its tool from the chest, reads `requests: none` and does its work;
  K proves the held tool survives a relaunch. `TOOL_SELFTEST=1` inverts the "held" read (and
  `SELFTEST=1` sets it together with the miner's depth inversion), so the assertion can fail.

**0.2.5** — the third slice, the two jobs' buildings:

- **`minecolonies:blockhutlumberjack`** (upstream's *Forester's Hut*) and
  **`minecolonies:blockhutminer`** (upstream's *Mine*): two more hut blocks with their own
  generated textures and slabs, recipes, loot tables and the axe tag, declared with the town hall's
  block properties (hardness 10, blast resistance infinite, wood sound, no occlusion).
- **A job needs its building, for every job.** `Job::building()` now answers the lumberjack's hut
  and the mine as well as the farmer's hut; `job assign … lumberjack|miner` is **refused** by name
  while the building is not standing inside the colony, and the job does not tick without it.
- **Breaking a hut takes its building with it**: a new `player_destroyed_block` hook drops the
  hut's marker, so `/minecolonies job info <citizen>` reads "lumberjack's hut missing" (previously
  it only ever reported the farmer's hut, and never noticed a break). It reads "no building" for an
  unemployed citizen.
- **Both new recipes** craft in a real table (`PCP/PBP/PPP`: planks, a crafting table where
  upstream's `structurize:sceptergold` goes, and the trade's wooden tool), asserted with their
  ingredients consumed and a wrong-ingredient control; `tools/craft_check.py` now covers all five
  carried recipes.
- The in-game leg gained L1–L3 and M1: the same four-log tree is left whole with the hut absent and
  reads 4 × air with 4 `oak_log` in the chest with it placed, and `HUT_SELFTEST=1` inverts the
  break assertion so it can fail.

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
