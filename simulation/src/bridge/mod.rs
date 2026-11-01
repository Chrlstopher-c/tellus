//! Pont napi vers le processus utilityProcess : exports, emplacement du monde courant, traduction JSON.
//! Aucune règle du monde ici (REGLES §13).

mod encode;
mod exports;
mod slot;

#[cfg(test)]
mod tests;

pub use exports::{
    create_world, drain_events, inspect, intervene, lookup_ids, lookup_weights, schedule, snapshot, step, vitals,
};
pub use slot::WorldSlot;
