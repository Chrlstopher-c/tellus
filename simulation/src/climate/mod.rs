//! Climat : orbite képlérienne, insolation de Berger, corps réels (planets.json), EBM Budyko–Sellers implicite,
//! cycle de l'eau diagnostique (vents, évaporation, précipitations orographiques, humidité du sol, glace de mer).

pub mod body;
mod ebm;
pub mod error;
pub mod greenhouse;
pub mod hydro;
pub mod ice;
pub mod insolation;
pub mod koppen;
pub mod mesh;
pub mod orbit;
pub mod orography;
pub mod solver;
pub mod state;
pub mod wind;

pub use error::ClimateError;
pub use hydro::HydroReport;
pub use state::{
    init, step, ClimateConfig, ClimateInputs, ClimateReport, ClimateState, EquilibriumReport, OlrLaw, Surface,
};
