//! Vivant (docs/REGLES.md § 7, docs/recherche/vivant.md) : flore, faune et évolution au-dessus de la matière.
//! Rien n'est préfait : le monde naît sans lignée ; seules `spawn_lineage` introduit du vivant.

pub mod evolution;
pub mod fauna;
pub mod flora;
pub mod graph;
pub mod io;
pub mod ledger;
pub mod naming;
pub mod regime;
pub mod state;

mod pipeline;

#[cfg(test)]
mod tests;

pub use io::{LifeInputs, LifeReport};
pub use ledger::{Flows, LifeLedger, Pools};
pub use pipeline::{step, PHASE_PERIOD};
pub use state::{init, spawn_lineage, LifeOutputs, LifeState, FOUNDER_N};
