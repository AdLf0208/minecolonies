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
