//! Mondes synthétiques pour les tests du vivant : grille réduite, forçages analytiques, lignées types.

use crate::clock::Regime;
use crate::life::evolution::{idx, Genome, NT};
use crate::life::{init, spawn_lineage, step, LifeInputs, LifeReport, LifeState};
use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;

pub const SEED: u64 = 42;

pub fn grid(level: u8) -> Grid {
    Grid::new(level, R_EARTH_M)
}

/// Forçages possédés par le test ; `inputs()` les emprunte au format du contrat.
pub struct Forcing {
    temperature_c: Vec<f32>,
    precip_mm_yr: Vec<f32>,
    soil_moisture: Vec<f32>,
    ice_fraction: Vec<f32>,
    is_ocean: Vec<bool>,
    npp_cap_gc_m2_yr: Vec<f32>,
    n_avail: Vec<f32>,
    p_avail: Vec<f32>,
    nutrient_ocean: Vec<f32>,
}

impl Forcing {
    pub fn inputs(&self) -> LifeInputs<'_> {
        LifeInputs {
            temperature_c: &self.temperature_c,
            precip_mm_yr: &self.precip_mm_yr,
            soil_moisture: &self.soil_moisture,
            ice_fraction: &self.ice_fraction,
            is_ocean: &self.is_ocean,
            npp_cap_gc_m2_yr: &self.npp_cap_gc_m2_yr,
            n_avail: &self.n_avail,
            p_avail: &self.p_avail,
            nutrient_ocean: &self.nutrient_ocean,
            o2_fraction: 0.21,
            co2_ppm: 280.0,
        }
    }
}

/// Monde terrestre analytique : température et pluie décroissent avec la latitude, aucun océan.
pub fn temperate_forcing(grid: &Grid) -> Forcing {
    let lat: Vec<f64> = grid.centers().iter().map(|c| c[2].clamp(-1.0, 1.0).asin()).collect();
    let temperature_c: Vec<f32> = lat.iter().map(|l| (28.0 - 40.0 * l.sin().powi(2)) as f32).collect();
    let precip_mm_yr: Vec<f32> = lat
        .iter()
        .map(|l| (200.0 + 1200.0 * (-(l / 0.5).powi(2)).exp()) as f32)
        .collect();
    let n = lat.len();
    Forcing {
        soil_moisture: precip_mm_yr.iter().map(|p| (p / 1500.0).clamp(0.0, 1.0)).collect(),
        precip_mm_yr,
        temperature_c,
        ice_fraction: vec![0.0; n],
        is_ocean: vec![false; n],
        npp_cap_gc_m2_yr: vec![3000.0; n],
        n_avail: vec![12.0; n],
        p_avail: vec![1.5; n],
        nutrient_ocean: vec![0.5; n],
    }
}

pub fn world(level: u8) -> (Grid, LifeState, Forcing) {
    let grid = grid(level);
    let state = init(&grid);
    let forcing = temperate_forcing(&grid);
    (grid, state, forcing)
}

/// Un pas annuel (Δt = 1 an, régime explicite) au tick `tick`.
pub fn run(state: &mut LifeState, grid: &Grid, forcing: &Forcing, tick: u64) -> LifeReport {
    step(state, grid, &forcing.inputs(), 1.0, Regime::Annual, tick, SEED)
}

/// Communauté ensemencée : 24 herbivores (masses et optimums thermiques étalés), 8 carnivores, des producteurs.
/// Chaque lignée est introduite en un point ; la dispersion la répand ensuite.
pub fn seed_community_wide(state: &mut LifeState, cells: usize) {
    spawn_lineage(state, producer_traits(), cells / 3).expect("cellule valide");
    for k in 0..24_u32 {
        let mut g = herbivore_traits();
        g[idx::LOG_MASS] = 0.2 + 0.5 * f64::from(k) / 24.0;
        g[idx::T_OPT] = (f64::from(k) + 0.5) / 24.0;
        spawn_lineage(state, g, (k as usize * 97 + 11) % cells).expect("cellule valide");
    }
    for k in 0..8_u32 {
        let mut g = carnivore_traits();
        g[idx::LOG_MASS] = 0.5 + 0.4 * f64::from(k) / 8.0;
        g[idx::T_OPT] = (f64::from(k) + 0.5) / 8.0;
        spawn_lineage(state, g, (k as usize * 131 + 50) % cells).expect("cellule valide");
    }
}

/// Lignée photosynthétique (trait 17 = 1) : seule source de flore.
pub fn producer_traits() -> Genome {
    let mut g = [0.5; NT];
    g[idx::PHOTO] = 1.0;
    g
}

/// Herbivore de petite taille, généraliste thermique, grégaire.
pub fn herbivore_traits() -> Genome {
    let mut g = [0.5; NT];
    g[idx::LOG_MASS] = 0.4;
    g[idx::DIET] = 0.0;
    g[idx::T_OPT] = 0.5;
    g[idx::T_WIDTH] = 0.3;
    g[idx::MOBILITY] = 0.3;
    g[idx::REPRO] = 0.5;
    g[idx::PHOTO] = 0.0;
    g
}

/// Carnivore de taille moyenne, chasseur solitaire.
pub fn carnivore_traits() -> Genome {
    let mut g = herbivore_traits();
    g[idx::LOG_MASS] = 0.7;
    g[idx::DIET] = 1.0;
    g[idx::HUNT] = 0.5;
    g
}

/// Lignée à tous les traits égaux (test de bornes).
pub fn uniform_traits(value: f64) -> Genome {
    [value; NT]
}
