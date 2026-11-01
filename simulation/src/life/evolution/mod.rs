//! Évolution : génome (18 traits), lignées et phylogénie, extinction, spéciation allopatrique.

pub mod extinction;
pub mod genome;
pub mod lineage;
pub mod speciation;

pub use genome::{
    adult_mass_kgc, birth_fraction, genetic_distance, idx, longevity_yr, mobility_rate, mutate, repro_fraction, Genome,
    NT,
};
pub use lineage::{refresh_moments, Lineage, LineageId, LineageTable, PhyloNode};
