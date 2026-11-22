# VALIDATION — MineColonies slices 1, 2, 3, 4 and 5

Validated on Linux with Rust 1.97/1.99 against **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20) and horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, through `ports/tools/build-slot.sh`
(`CARGO_INCREMENTAL=0`, no bare `cargo`, no `--target-dir`).

## 0.2.7 — slice 5, the food request (2026-11-22)

| Check | Command | Result |
|---|---|---|
| rules first (red) | `cargo test --offline`, the new rules as `panic!()`/`unimplemented!()` bodies | **71 passed, 9 failed** — `a job's food` / `not implemented: a work tick's saturation` / `not implemented: a hungry citizen` / `not implemented: the citizen's saturation` / the request kind / the record's saturation / the support round trip (`sdk-test-slice5-red.log`) |
| tests | `cargo test --offline --locked` | **80 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | exit 0 (`sdk-fmt.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2, `cmp` | `CMP:identical`, **81,477 B**, sha256 `63825cc5327a584ca837f0f2d0ee428cfc737d11122a993645a2e869aea0ac92` (`sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log`) |
| registry check | `cargo horizon-mod check minecolonies.hmod` | `minecolonies 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`; 81,477 B, sha `63825cc5…` (`sdk-check.log`, run against that sha) |
| source compile | `cargo horizon-mod check --uses minecolonies.hmod` | exit 0 (`sdk-check-uses.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, sha `63825cc5…` (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, sha `63825cc5…` (`sdk-fits-other.log`) |
| content | `python3 tools/gen_assets.py --check` + `tools/assemble_content.py --check` | **25** assets byte-identical, `data/` matches (`sdk-assets.log`) |
| no jar-derived asset | `python3 tools/check_provenance.py` | tree 25 files / 14,006 B; **jar-derived asset bytes: 0**, files matched 0, over 22,747 jar files (20,465 distinct hashes; 8,470 under `assets/minecolonies/**`) (`sdk-provenance.log`) |
| package contents | unzip listing | **53 files**, `LICENSE` + `LICENSE-NOTICE.md`, no `Cargo.lock` (`sdk-entries.txt`) |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` (0.2.7, so no cached-package reuse) |
| in game | `build-slot.sh run … --step e2e -- bash tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–O (`minecolonies-e2e/e2e-launcher.log`) |
| control | `GAME=$CONTROL_GAME … tools/e2e.sh` | **FAIL** at `ctl place of minecolonies:blockhuttownhall … failed` / `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` (`e2e-launcher-control.log`) |
| fail-capability (the meal) | `SATIETY_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: Bo reads full saturation, so the meal was applied`, `EXIT:1` (`e2e-satiety-selftest.log`) |
| fail-capability (the hand) | `TOOL_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: Ada holds minecolonies:scythe, so the hand was written`, `EXIT:1` (`e2e-tool-selftest.log`) |
| fail-capability (hand + depth) | `SELFTEST=1 … tools/e2e.sh` | **FAIL** at the same held read, `EXIT:1` (`e2e-selftest.log`) |
| fail-capability (the break) | `HUT_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the broken hut reads missing, so the marker went with the block`, `EXIT:1` (`e2e-hut-selftest.log`) |
| fail-capability (founding) | `FOUNDING_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the second town hall was refused, as it should be`, `EXIT:1` (`e2e-founding-selftest.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=9 ok=9 missing-exit=0 bad-exit=0`, exit 0 |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | `missing_families=0 … missing=0`, exit 0; `food_request` and `citizen saturation` are `tested` rows |

The in-game leg's new assertions, from `e2e-launcher.log`:

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

The tree stands 18 blocks from Bo — outside the lumberjack's 16-block search — so it cannot fall
while the saturation is read; after the meal the citizen is moved beside it, which is what makes the
last assertion about the job. `SATIETY_SELFTEST=1` inverts the "fed" read, so a port whose meal was
never applied passes the inverted leg and fails the normal one.

## 0.2.6 — slice 4, the tool request (2026-11-22)

| Check | Command | Result |
|---|---|---|
| rules first (red) | `cargo test --offline`, the new rules as `panic!()`/`unimplemented!()` bodies | **59 passed, 9 failed** — `a job's tool` / `not implemented: a job's tool` / `not implemented: the citizen's hand` / `not implemented: the colony's requests` / the record field / the support round trip (`sdk-test-slice4-red.log`) |
| tests | `cargo test --offline --locked` | **68 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | exit 0 (`sdk-fmt.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2, `cmp` | `CMP:identical`, **77,378 B**, sha256 `bc41900e63fe6e766a6c188a065c865cc8dd49deda8b68ca7e0dcc9dd0f947b6` (`sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log`) |
| registry check | `cargo horizon-mod check minecolonies.hmod` | `minecolonies 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`; 77,378 B, sha `bc41900e…` (`sdk-check.log`) |
| source compile | `cargo horizon-mod check --uses minecolonies.hmod` | exit 0 (`sdk-check-uses.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, sha `bc41900e…` (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, sha `bc41900e…` (`sdk-fits-other.log`) |
| content | `python3 tools/gen_assets.py --check` + `tools/assemble_content.py --check` | **25** assets byte-identical, `data/` matches (`sdk-assets.log`) |
| no jar-derived asset | sha256 of every `assets/` file against every jar entry | tree 25 files / 14,006 B; **jar-derived asset bytes: 0**, files matched 0, over 24,101 jar entries (20,465 distinct hashes) (`sdk-provenance.log`) |
| package contents | unzip listing | **53 files**, `LICENSE` + `LICENSE-NOTICE.md`, no `Cargo.lock` (`sdk-entries.txt`) |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` (0.2.6, so no cached-package reuse) |
| in game | `build-slot.sh run … --step e2e -- bash tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–N (`minecolonies-e2e/e2e-launcher.log`) |
| control | `GAME=$CONTROL_GAME … tools/e2e.sh` | **FAIL** at `ctl place of minecolonies:blockhuttownhall … failed` / `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` (`e2e-launcher-control.log`) |
| control inside one run | `CONTROL=1 … tools/e2e.sh` | control leg *"without the package: no town hall item"*, then modded **PASS** `EXIT:0` (`e2e-launcher-control-inside.log`) |
| fail-capability (the hand) | `TOOL_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: Ada holds minecolonies:scythe, so the hand was written`, `EXIT:1` (`e2e-tool-selftest.log`) |
| fail-capability (hand + depth) | `SELFTEST=1 … tools/e2e.sh` | **FAIL** at the same held read, `EXIT:1` (`e2e-selftest-tool-and-depth.log`) |
| fail-capability (depth) | `MINER_SELFTEST=1 … tools/e2e.sh` | **FAIL** `the miner did not dig 9 blocks above the bedrock`, `EXIT:1` (`e2e-selftest.log`) |
| fail-capability (break) | `HUT_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the broken hut reads missing, so the marker went with the block`, `EXIT:1` (`e2e-hut-selftest.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=8 ok=8 missing-exit=0 bad-exit=0`, exit 0 |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | `missing_families=0 … missing=0`, exit 0 |

The in-game leg's new assertions, from `e2e-launcher.log`:

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

## 0.2.5 — slice 3, the two jobs' buildings (2026-11-21)

| Check | Command | Result |
|---|---|---|
| rules first (red) | `cargo test --offline`, the new rules as `panic!()` bodies | **54 passed, 4 failed** — `a job's building` / `not implemented: a job's building status` / `not implemented: the hut blocks` (`sdk-test-slice3-red.log`) |
| tests | `cargo test --offline --locked` | **59 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --offline --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | `FMT_EXIT:0` (`sdk-fmt.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2, `cmp` | `CMP:identical`, **71,966 B**, sha256 `9fbb989826e8b472ef026467286fc2af9b84b5b7134424a40e8235ba5043a635` (`sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log`) |
| registry check | `cargo horizon-mod check minecolonies.hmod` | `minecolonies 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`; 71,966 B, sha `9fbb9898…` (`sdk-check.log`, run against that sha) |
| source compile | `cargo horizon-mod check --uses minecolonies.hmod` | `USES_EXIT:0` (`sdk-check-uses.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, sha `9fbb9898…` (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, sha `9fbb9898…` (`sdk-fits-other.log`) |
| content | `python3 tools/gen_assets.py --check` + `tools/assemble_content.py --check` | **25** assets byte-identical, `data/` matches (`sdk-assets.log`) |
| no jar-derived asset | sha256 of every `assets/` file against every jar entry | tree 25 files / 14,006 B; **jar-derived asset bytes: 0**, files matched 0, over 22,747 jar entries (`sdk-provenance.log`) |
| package contents | unzip listing | **53 files**, `LICENSE` + `LICENSE-NOTICE.md`, no `Cargo.lock` (`sdk-entries.txt`) |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` |
| in game | `build-slot.sh run … --step e2e -- bash tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–N (`minecolonies-e2e/e2e-launcher.log`) |
| control | `GAME=$CONTROL_GAME … tools/e2e.sh` | **FAIL** `ctl place of minecolonies:blockhuttownhall … failed` / `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` (`e2e-launcher-control.log`) |
| control inside one run | `CONTROL=1 … tools/e2e.sh` | control leg *"without the package: no town hall item"*, then modded **PASS** `EXIT:0` (`e2e-launcher-control-inside.log`) |
| fail-capability (depth) | `SELFTEST=1 … tools/e2e.sh` | **FAIL** `the miner did not dig 9 blocks above the bedrock`, `EXIT:1` (`e2e-selftest.log`) |
| fail-capability (break) | `HUT_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the broken hut reads missing, so the marker went with the block`, `EXIT:1` (`e2e-hut-selftest.log`) |
| fail-capability (founding) | `FOUNDING_SELFTEST=1 … tools/e2e.sh` | **FAIL** `selftest: the second town hall was refused, as it should be`, `EXIT:1` (`e2e-founding-selftest.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=6 ok=6`, exit 0 |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | `missing_families=0 … missing=0`, exit 0 |

The in-game leg's new assertions (L1–L3, M1), from `e2e-launcher.log`:

```
== L1: the lumberjack's hut is its building: without it the assignment is refused ==
  without its hut: A lumberjack needs lumberjack's hut in the colony: place minecolonies:blockhutlumberjack.
== L2: the hut is placed, job info reads it, and the tree falls ==
  Bo: job Lumberjack, colony Riverside (3 crops harvested), 0 logs felled, 0 blocks mined, lumberjack's hut present.
  after the fell: soil at 2004,101,-1 is 'minecraft:dirt', stump is 'minecraft:oak_sapling',
    chest [{"count":4,"item":"minecraft:oak_log"}]
== L3: breaking the hut takes the building with it ==
  after the break: Bo: job Lumberjack, …, lumberjack's hut missing.
== M1: the mine is the miner's building ==
  Cass: job Miner, …, mine present.
  the shaft is 1x1 and 8 deep, the bedrock untouched, 8 cobblestone in the chest
== N: the carried recipes craft, and the carried items are real ids ==
  lumberjack's hut: planks around a crafting table and a wooden axe -> {"count": 1, "item": "minecolonies:blockhutlumberjack", …}
  mine: planks around a crafting table and a wooden pickaxe -> {"count": 1, "item": "minecolonies:blockhutminer", …}
  lumberjack's hut with a stick where the axe goes -> null
```

## 0.2.4 — slice 2, the `lumberjack` and `miner` jobs (2026-11-21)

| Check | Command | Result |
|---|---|---|
| repin | `cargo horizon-mod update --horizon 0a875da…` | *"none of the 287 items it uses changed"*, pinned; `.horizon/sdk` refetched |
| runtime first (red) | `cargo test --offline`, the three new rules as `unimplemented!()` | **41 passed, 14 failed** — `not implemented: the tree's connected cluster` / `the miner's shaft` (`sdk-test-slice2-red.log`) |
| tests | `cargo test --offline --locked` | **55 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --offline --locked --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | `FMT_EXIT:0` (`sdk-fmt.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2 | `CMP:identical`, **63,940 B**, sha256 `d81dcd77b30d37f2c0291ee2b4ee736e03848754d04661f31dc7493960dac0aa` (`sdk-build.log`, `sdk-build-second.log`, `sdk-sha256.log`) |
| registry check | `cargo horizon-mod check <delivered file>` | `minecolonies 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`; 63,940 B, same sha (`sdk-check.log`) |
| source compile | `cargo horizon-mod check --uses <delivered file>` | exit 0 (`sdk-check-uses.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for` (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits` (`sdk-fits-other.log`) |
| content | `python3 tools/gen_assets.py --check` + `tools/assemble_content.py --check` | 15 assets byte-identical, `data/` matches (`sdk-assets.log`) |
| package contents | unzip listing | **37 entries**, `LICENSE` + `LICENSE-NOTICE.md`, no `Cargo.lock` (`sdk-entries.txt`) |
| gate | `python3 ports/tools/verify-sdk-package.py minecolonies` | **1/1 package(s) verified** |
| game build | `ports/tools/launcher-game.sh build minecolonies` | `Compiling minecolonies (1 of 1)` |
| in game | `build-slot.sh run … --step e2e -- tools/e2e.sh` | **PASS**, `EXIT:0`, phases A–N (`minecolonies-e2e/e2e-launcher.log`) |
| control | `GAME=$CONTROL_GAME tools/e2e.sh` | **FAIL** at `Unknown item 'minecolonies:blockhuttownhall'`, `EXIT:1` (`e2e-launcher-control.log`) |
| control inside one run | `CONTROL=1 tools/e2e.sh` | control leg *"without the package: no town hall item"*, then modded **PASS** `EXIT:0` (`e2e-launcher-control-inside.log`) |
| fail-capability (depth) | `SELFTEST=1 tools/e2e.sh` | **FAIL** `the miner did not dig 9 blocks above the bedrock`, `EXIT:1` (`e2e-selftest.log`) |
| fail-capability (founding) | `FOUNDING_SELFTEST=1 tools/e2e.sh` | **FAIL** `selftest: the second town hall was refused, as it should be`, `EXIT:1` (`e2e-founding-selftest.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py minecolonies-e2e --strict` | `logs=5 ok=5`, exit 0 |
| coverage | `python3 ports/tools/check-coverage.py minecolonies` | `missing_families=0 missing=0`, exit 0 |

The in-game leg's own new assertions (phases L–N), from `e2e-launcher.log`:

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
== N: the carried recipes craft, and the carried items are real ids ==
  town hall: 8 planks around a crafting table -> {"count": 1, "item": "minecolonies:blockhuttownhall", …}
  farmer's hut: 8 planks around an iron hoe -> {"count": 1, "item": "minecolonies:blockhutfarmer", …}
  scythe: two iron ingots with a gap over two sticks -> {"count": 1, "item": "minecolonies:scythe", …}
  scythe with a stick where the iron goes -> null
  the scythe is real, an unknown id is refused: Unknown item 'minecolonies:no_such_item'
```

The leg launches `--offscreen` (phase N opens a crafting table's menu; a menu needs a screen).

## History — 0.1.x `.hnmod` (superseded)

> **History only.** This file records the 0.1.x `.hnmod` ladder (§1–§6). The 0.2.0 flat SDK package
> (pin `0b14f8a`, 2026-11-18) is verified by `notes.md` §"Verification" and by the logs
> `ports/_shared/evidence/minecolonies/sdk-{test,clippy,fmt,build,check,content}.log`; the runtime
> tests are the same 36, and `src/runtime.rs` is byte-identical to the bundle these lines built.
> **No in-game run exists for 0.2.0** (no game binary at `0b14f8a`); the e2e logs below are 0.1.x.

## 1. Runtime, before any implementation (red)

```
$ cargo test --offline
test result: FAILED. 0 passed; 36 failed; 0 ignored; 0 measured; 0 filtered out
```
All 36 failed on `not implemented` (`unimplemented!()` bodies), which is the red TDD wants: each
test names the behaviour it pins, and none passed early.

## 2. Runtime, after implementation (green)

```
$ cargo test --offline
running 36 tests
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 3. Hand-back checks

```
$ rustfmt --edition 2024 --check mixins/minecolonies/*.rs
FMT ok
$ cargo clippy --all-targets --offline -- -D warnings
CLIPPY ok            (0 warnings)
```

## 4. The staged ladder (Horizon pin `1d63e0a`)

```
$ python3 tools/horizon.py stage    # staged at the pin
$ python3 tools/horizon.py check    # cargo_exit: 0, 0 warnings in the injected module
$ python3 tools/horizon.py test     # game::minecolonies::tests: 36 passed (1240 filtered out), cargo_exit 0
$ python3 tools/horizon.py build    # cargo_exit: 0, binary target/horizon-cargo/fast/horizon-client
```
The fresh logs for these runs are in
`ports/_shared/evidence/minecolonies/ladder-{fmt,clippy,unit,embedded-test,staged-check}.log`
and `package.log`. The dev-mod, embedded and selftest runs below were made from this same build, in
one lease.
```

## 5. Content and package

```
$ python3 tools/gen_assets.py --check
assets/minecolonies/: 15 files, byte-identical to the palette's output
$ python3 tools/assemble_content.py --check
assets/minecolonies/: 15 files, byte-identical to the palette's output      (exit 0)
$ tools/package.sh
content/minecolonies.hmod  size=8204    sha256=bf84ad07565a0d543da5267dffb87768c702c3fcb146107d671a7062cfe34736 entries=28
minecolonies.hnmod         size=38746   sha256=3e6c4ba850360da0a1ca452239285972e603f03e3a184280e7c1f97dd739c0b6 entries=15
$ python3 tools/package.py      # a second run, independently
"sha256": "3e6c4ba850360da0a1ca452239285972e603f03e3a184280e7c1f97dd739c0b6"   (same bytes)
```
The archive carries `mixins/minecolonies/LICENSE` (35,121 B, the GPL-3.0 text) and
`mixins/minecolonies/LICENSE-NOTICE.md`.

## 6. In-game evidence

| Log | Run | Result |
|---|---|---|
| `ports/_shared/evidence/minecolonies/e2e-dev.log` | `tools/e2e.sh` (dev-mod) | `PASS` + `EXIT:0` |
| `ports/_shared/evidence/minecolonies/e2e-embedded.log` | `EMBEDDED=1` (no `--dev-mod`, with the `.horizon-cache` control) | `PASS` + `EXIT:0` |
| `ports/_shared/evidence/minecolonies/e2e-selftest.log` | `SELFTEST=1` | `FAIL` + `EXIT:1` |

The dev-mod log's assertions, in order:

```
== A: the town hall founds a colony ==
  Colony Player…'s Colony founded: 64 blocks around the town hall, owner Player…
  Colony Player…'s Colony (id 1, owner Player…): centre 2000,101,0, 64 blocks (4 chunks),
  0 citizens, 0 crops harvested, founded at tick N.
== B: the colony is renamed ==           Colony Riverside (id 1, …)
== C: a second town hall 120 blocks away is refused ==
  refusal: No colony here: too close to Riverside: 120 blocks, the closest a colony may stand is 8 chunks.
== D: 120 blocks across and 120 up is refused too (the distance is horizontal) ==   (same line)
== E: a town hall 200 blocks away is refused ==
  refusal: No colony here: you already own Riverside.
== G: a citizen joins the colony ==      Ada joined Riverside (1 citizens) …; 1. Ada — Unemployed
== H: the job needs its building ==      Ada: job Unemployed, …, farmer's hut present; Ada is now the colony's farmer; 1. Ada — Farmer
== I: the farmer works ==                harvested 0 -> 1, near crop age 7 -> 0;
                                         citizen 2002.5 -> 2005.44, far crop age 7 -> 0
== J: a citizen with no colony does nothing ==
  the citizen with no colony left the crop at age 7 and stayed at x=2100.5
== K: the colony survives quit and relaunch ==
  after the relaunch: 2 -> 3 harvested, roster intact
PASS: …
EXIT:0
```

The embedded log runs the control first:

```
== control: the same game with no embedded content file ==
  without the embedded file: no town hall item, the block stayed minecraft:air
  embedded content: 8204 B at …/.horizon-cache/minecolonies-embedded-content.hmod,
  identical to content/minecolonies.hmod
```

## 7. Gates

```
$ python3 ports/tools/check-e2e-logs.py minecolonies --strict   # exit 0
$ python3 ports/tools/check-skills.py --notes ports/minecolonies # PASS
```

## 6. Re-pin to the Nightly `9f42064` (2026-11-13) — supersedes §4–§5

Pin `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, horizon-mixins unchanged
(`5a64af938ec86b00d53d16a408aabc0038fb3e57`, schema 6). The Nightly's
`game::mods::content_root` took a fourth parameter (`disabled`), so the embedded-content binding
gained it (`notes.md`, "Re-pin to the Nightly"): one binding, one helper parameter, both
`horizon_revision` fields and the `game.rs` hash.

```
# RED, before the binding: the defect this slice removes
$ python3 tools/horizon.py check
error[E0061]: this function takes 4 arguments but 3 arguments were supplied
   --> crates/horizon-client/src/game/mods.rs:123:43
cargo_exit: 101                        evidence/minecolonies/nightly-check-red.log

# with the fourth binding
$ python3 tools/horizon.py stage     # plan + apply green, schema 6, 9f42064…
$ python3 tools/horizon.py check     # cargo_exit: 0, 0 warnings   nightly-check.log
$ python3 tools/horizon.py test      # game::minecolonies::tests: 36 passed, 0 warnings
                                     # (2022 filtered), cargo_exit 0   nightly-test.log
$ python3 tools/horizon.py build     # cargo_exit: 0, binary fast/horizon-client 128,315,344 B
                                     # nightly-build.log
```

The Nightly's lock needs one online `cargo fetch --locked` in the staged tree before the first
offline build (`cargo_exit 101, no matching package named 'framehop'` otherwise).

```
$ tools/package.sh                   # twice, byte-identical
content/minecolonies.hmod  size=8204    sha256=bf84ad07565a0d543da5267dffb87768c702c3fcb146107d671a7062cfe34736 entries=28
minecolonies.hnmod         size=38827   sha256=dc665016287c45206f14bb9d1466fd85ac82f0b9e5970159331b9f11c5c87f79 entries=15
$ python3 tools/package.py           # second run: the same 38,827 B / dc665016…
```

Only three archive entries differ from the previous bundle: `horizon-native.json`, the
`embedded_content.rs` helper, and `horizon-mixin.toml`; the content `.hmod` and
the embedded copy are byte-identical to each other and to the previous build.

```
$ EMBEDDED=1 build-slot.sh e2e --job … --port ports/minecolonies -- tools/e2e.sh
== control: the same game with no embedded content file ==
  without the embedded file: no town hall item, the block stayed minecraft:air
  embedded content: 8204 B at …/.horizon-cache/minecolonies-embedded-content.hmod,
  identical to content/minecolonies.hmod
…  colony founded / two refusals / citizen hired / farmer harvested and walked /
   no-colony control inert / 2 -> 3 harvested after relaunch
PASS: …
EXIT:0                               evidence/minecolonies/e2e-nightly.log
```

Run twice: once with a `horizon-ctl` built at this pin
(`sha256 7c192508…`, `/tmp/opencode/minecolonies-nightly-ctl/`), once with `e2e.sh` as shipped and
its default ctl (the shared `horizon-1d63e0a5` pair). Both `PASS` + `EXIT:0`, so the game half does
not depend on the ctl's pin (the ctl protocol is additive). The log now ends `EXIT:0` followed by
the slot runner's own release line.

## 7b. Gates at the Nightly

```
$ python3 ports/tools/check-e2e-logs.py minecolonies --strict   # exit 0 (4 logs)
$ python3 ports/tools/check-skills.py --notes ports/minecolonies # PASS
$ python3 ports/tools/upstream_ready.py minecolonies --copyleft  # exit 1, ONE blocker:
  BLOCKED minecolonies GPL3
    - pinned to 9f42064c11ef, not 1d63e0a5
```
That blocker is the campaign pin hardcoded in `ports/tools/upstream_ready.py:66`; a bundle that
names `content_root`'s `disabled` parameter cannot also be pinned at `1d63e0a`. Campaign ruling,
not a port defect.
