//! Domaine `matter` : cycles de l'eau, du carbone, de l'azote, du phosphore et de l'oxygène (REGLES § 6, nutriments).
//!
//! Chaque stock ne change que par le registre `shared::ledger` (flux limité au stock source). La biologie tourne en
//! 12 sous-pas quand Δt ≤ 1 an, et en un pas à relaxation exacte au-delà ; la géologie tourne par sous-pas ≤ 100 ans.
//! Contrat d'intégration : `init(config, grid, inputs)` puis `step(state, grid, inputs, dt, regime, tick, seed)`.

mod carbon;
mod error;
mod forcing;
mod geology;
mod inputs;
mod nitrogen;
mod oxygen;
mod params;
mod phosphorus;
mod report;
mod state;
#[cfg(test)]
mod tests;
mod tick;
mod water;

pub use error::MatterError;
pub use inputs::MatterInputs;
pub use params::MatterConfig;
pub use report::{ElementDrift, MatterReport};
pub use state::{init, MatterState};
pub use tick::step;
