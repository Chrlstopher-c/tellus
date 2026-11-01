//! Orchestrateur du monde (REGLES §12) : possède grille, planète, climat, matière, horloge, table de rendu et journal.
//! Le pont (`bridge/`) n'appelle que ce module ; aucune règle physique n'y est écrite, seulement l'ordre des domaines.

mod create;
mod error;
mod journal;
mod lookup;
mod name;
mod physics;
mod request;
mod schedule;
mod state;
#[cfg(test)]
mod tests;

pub use error::WorldError;
pub use journal::WorldEvent;
pub use lookup::{Lookup, LOOKUP_HEIGHT, LOOKUP_WIDTH};
pub use name::name_from_seed;
pub use request::{GeneratedSpec, Source, WorldRequest, DEFAULT_LEVEL, MAX_LEVEL};
pub use state::{CellDetail, Vitals, World, WorldInfo};
