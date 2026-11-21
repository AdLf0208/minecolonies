//! MineColonies for Horizon — slices 1–4: a town hall founds a colony, one citizen joins it and
//! farms, a lumberjack and a miner work their own buildings, each worker's trade tool is a
//! request the colony's chest fulfils, and `/minecolonies colony|citizen|job|requests` is the
//! configuration surface.
//!
//! The one `impl Hooks` at the crate root: every method is a *moment* the game tells mods
//! about, and each forwards to `support` (the game-side glue) or, through it, to `runtime` (the
//! pure colony rules, which have no `mod_api` type and are tested by `cargo test` alone).
//!
//! | moment | what this port does |
//! |---|---|
//! | `register` | declares the hut block entity types; attaches the hall's and the citizen's Parts |
//! | `entity_place` | refuses a town hall that may not found a colony (the only cancel point) |
//! | `level_tick_post` (server) | completes a placement, takes a requested tool, applies deliveries |
//! | `register_commands` | `/minecolonies …` and the mod's own `mc` alias |
//! | `run_mod_command` | runs a parsed `/minecolonies …` usage, `requests` included |
//!
//! The citizen's job is **not** a global hook: it is the `EntityPart` in `parts`, called for
//! `minecolonies:citizen` rows only. The colony is **not** a global scan either: it is one
//! attachment per town hall block entity (`support::COLONY`), which is what the SDK exposes
//! instead of an enumeration of the world's block entities.
use mod_api::{
    self, EntityPlaceEvent, Flow, Hooks, HostSide, LevelTickPostEvent, PlayerDestroyedBlockEvent,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
};

#[cfg(not(target_arch = "wasm32"))]
mod parts;
mod runtime;
mod support;

/// The mod: a marker type, one instance, every method stateless (the colony lives in the world).
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declares the hut block entity types and attaches the citizen's Part, before any world
    /// opens.
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        support::register(r);
    }

    /// A block is about to be placed (`BlockEvent.EntityPlaceEvent`). A town hall that could not
    /// found a colony is refused here, because this is the only moment that can still stop the
    /// write; a legal one is queued and completed at that level's `level_tick_post`, once the block exists.
    fn entity_place(&self, host: &mut dyn ServerHost, event: &mut EntityPlaceEvent) -> Flow {
        support::on_entity_place(host, event)
    }

    /// The end of a level's tick, on the server: completes the placements queued in *that* level
    /// (the colony record, the hut marker). A level-scoped hook, not `server_tick`, because the
    /// colony is written through the level's own attachment store.
    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        if let HostSide::Server(host) = host {
            support::on_level_tick_post(host, event.level);
        }
    }

    /// A player's break removed a block: a hut's marker goes with it, so a broken building is
    /// `missing` to `job info` and its job stops (upstream's building is the job's own block).
    fn player_destroyed_block(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerDestroyedBlockEvent,
    ) {
        support::on_player_destroyed_block(host, event);
    }

    /// `/minecolonies …` (and the mod's `mc` alias), plus the `citizens` spelling it also uses.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::on_register_commands(event);
    }

    /// A parsed `/minecolonies …` usage: resolved arguments, no string splitting.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::on_run_mod_command(host, event)
    }
}
