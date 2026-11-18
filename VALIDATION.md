# VALIDATION — MineColonies slice 1

Every line is a command and its result, run in `ports/minecolonies` through the campaign's build
slots (`ports/tools/build-slot.sh`, `BUILD_LOCAL=1`: the second machine is unreachable).

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
