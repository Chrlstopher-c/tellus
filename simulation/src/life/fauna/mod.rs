//! Faune : cohortes par cellule, métabolisme MTE, consommation de Holling, reproduction, mortalité, dispersion.

pub mod cell;
pub mod cohort;
pub mod damuth;
pub mod dispersal;
pub mod dynamics;
pub mod feeding;
pub mod metabolism;

pub use cell::{update_cell, FaunaEnv, FaunaFlux};
pub use cohort::{fuse_to_cap, remove_lineages, Cohort, MAX_COHORTS};
