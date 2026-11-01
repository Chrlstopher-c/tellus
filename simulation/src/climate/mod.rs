//! Climat : orbite képlérienne, insolation de Berger, corps réels (planets.json), EBM Budyko–Sellers implicite.

pub mod body;
mod ebm;
pub mod error;
pub mod greenhouse;
pub mod ice;
pub mod insolation;
pub mod mesh;
pub mod orbit;
pub mod solver;
pub mod state;

pub use error::ClimateError;
pub use state::{
    init, step, ClimateConfig, ClimateInputs, ClimateReport, ClimateState, EquilibriumReport, OlrLaw, Surface,
};
