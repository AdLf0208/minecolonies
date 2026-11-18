# MineColonies — a Horizon port

**MineColonies** is a Minecraft mod. Minecolonies minecraft mod. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [ldtteam/minecolonies](https://github.com/ldtteam/minecolonies) — licence **GPL-3.0-only (code); assets ARR — regenerated, never copied**.

| | |
|---|---|
| Upstream mod | MineColonies ([source](https://github.com/ldtteam/minecolonies)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `minecolonies.hmod` (install it with the Horizon launcher) |
| Licence | **GPL-3.0-only (code); assets ARR — regenerated, never copied** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What this slice does

- **Founding.** Placing `minecolonies:blockhuttownhall` founds a colony named `<you>'s Colony`
  with you as its owner, a 64-block (4-chunk) radius, and the whole record saved with the town
  hall. Placing it where a colony would be too close (under
  `max(minColonyDistance, initialColonySize) × 16` = 128 blocks, measured horizontally), or where
  you already own a colony, is refused before the block appears. `/minecolonies colony rename`,
  `… colony radius <chunks>`, `… colony info`.
- **A citizen.** `/minecolonies citizen hire [name]` recruits a citizen into the colony;
  `/minecolonies citizen list` shows the roster.
- **A job.** `/minecolonies job assign <citizen> farmer` gives a citizen the farmer's job (it needs
  a `minecolonies:blockhutfarmer` in the colony). The farmer walks to a mature crop inside the
  colony every 40 ticks, harvests it and leaves it replanted; the colony counts the crop. A citizen
  with no colony does nothing.
- **Persistence.** The colony, its roster and the harvest count are saved with the world, and the
  job resumes after a relaunch.

## Not yet ported

- The other ~40 jobs (builder, miner, guard, crafter, …), the builder and its schematics
- The citizen's own model and texture (a native renderer slice); the scythe is a plain item with no
- The colony banner, the supply camp/ship, and every other block of the mod.
- **Not verified in game at this pin.** No game binary exists at `0b14f8a` (the shared pairs are
- A hut placed **before** this port's attachments existed is invisible to
- The villager-fallback citizen does not farm (above).
- `python3 ports/tools/upstream_ready.py minecolonies --copyleft` now reports the pin blocker

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `minecolonies.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

MineColonies was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **GPL-3.0-only (code); assets ARR — regenerated, never copied**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/ldtteam/minecolonies

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run minecolonies` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/minecolonies/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
