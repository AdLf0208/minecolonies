# Working on a native Horizon mod

> **Build slots (v2, 2026-11-09):** every build/test/e2e goes through the
> campaign runner (`ports/tools/build-slot.sh` from the campaign root,
> `../tools/build-slot.sh` from this port). It **blocks in the kernel** until a
> slot is free — a freed slot is taken in ~0.07 s — so **never poll**: no
> `while ! slot; sleep` loop, no background build. It sets `CARGO_BUILD_JOBS` per
> step (build/test 3, stage 2, e2e 1), journals grants/releases, and a timeout
> exits **75**. Read `../_shared/build-slots-usage.md` before running anything;
> observe with `status` / `watch` / `events` / `stats`, never with `ps`+`sleep`.
> Never pass `--target-dir`, never run a bare `cargo` here.

Read README.md and compatibility.json before editing. Keep this repository
separate from Horizon and horizon-mixins. The crate is a **flat SDK package**
(mod API `0.2`): `src/lib.rs` is the crate root, `src/runtime.rs` is the pure
logic and must stay byte-identical unless a slice deliberately changes a rule,
and `src/support.rs`/`src/parts.rs` hold the glue and the `EntityPart`. Never
edit `assets/` or `data/` by hand — `tools/assemble_content.py` and
`tools/gen_assets.py` produce them, and both have a `--check`.

Validate with `cargo fmt --check`, `cargo clippy --offline --locked
--all-targets -- -D warnings`, `cargo test --offline --locked`, then
`cargo horizon-mod build` + `check` + `check --uses` + `fits` on the package,
and `tools/e2e.sh` when a game binary for `horizon-mod.toml`'s `[native]` pin
exists. Keep validation claims and supported revisions honest: `check` does not
compile the Rust, so a green `check` is not a green build. The `.hnmod` layout
(`mixins/`, `horizon-mixin.toml`, `tools/horizon.py`, `tools/package.py`) is
retired and archived under `../_shared/evidence/minecolonies/legacy/`.
