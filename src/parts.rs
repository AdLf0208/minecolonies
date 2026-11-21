//! The citizen as an `EntityPart`: the game calls it for `minecolonies:citizen` rows only, so the
//! job is one entity type's moment and not a global hook (the retired `.hnmod` used the global
//! `entity_tick_listened`/`entity_tick_post` pair for it).
use mod_api::{self, EntityPart, GeneralEntityTickEvent, ServerHost};

use crate::support;

/// The citizen's behaviour on its own entity type.
pub(crate) struct CitizenWorker;

#[mod_api::horizon_part]
impl EntityPart for CitizenWorker {
    /// The citizen's authoritative tick: one work step every `FARMER_DELAY_TICKS`, if it has a
    /// colony, a job and its building.
    fn general_entity_tick(
        &self,
        host: &mut dyn ServerHost,
        event: &mut GeneralEntityTickEvent<'_>,
    ) {
        support::on_citizen_tick(host, event.entity);
    }
}

/// The town hall's Part: a vanilla-family entity type (the citizen is a villager template)
/// hears no `general_entity_tick` at this pin — "vanilla families tick their own paths" — so
/// the colony's own block entity drives the citizens' job ticks. The hall ticks every tick and
/// steps its roster's citizens; each citizen still works only on its own every-40th tick.
pub(crate) struct TownHallWorker;

#[mod_api::horizon_part]
impl mod_api::BlockPart for TownHallWorker {}

#[mod_api::horizon_part]
impl mod_api::BlockEntityPart for TownHallWorker {
    fn block_entity_ticks(
        &self,
        _host: &dyn mod_api::Host,
        event: &mut mod_api::BlockEntityTicksEvent<'_>,
    ) {
        event.ticks = true;
    }

    fn block_entity_tick(
        &self,
        host: &mut dyn mod_api::ServerHost,
        event: &mut mod_api::BlockEntityTickEvent<'_>,
    ) {
        support::on_hall_tick(host, event.pos);
    }
}
