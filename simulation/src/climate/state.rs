//! État climatique et contrat d'intégration (REGLES §12) : `init`, `step`, régimes (§1) et équilibre borné.
//! Le champ `ClimateState` possède son modèle (géométrie et paramètres figés) derrière un `Arc`.
//! Les champs hydrologiques (précipitations, vents, humidité du sol) sont réservés et restent à zéro.

#[cfg(test)]
mod tests;

use super::body::Body;
use super::ebm::{self, Forcing, SubstepReport};
use super::error::ClimateError;
use super::greenhouse::{calibrate_tau, grey_surface_temp_k, tau_for_world, GreenhouseError};
use super::ice::budyko_free_albedo;
use super::insolation::{build_table, InsolationTable, MONTHS};
use super::mesh::Mesh;
use crate::clock::{regime, Regime, System};
use crate::shared::grid::Grid;
use crate::shared::units::WATER_VOLUMETRIC_HEAT_CAPACITY;
use std::sync::Arc;

/// Diffusion méridienne D, W m⁻² K⁻¹ (climlab, unique pour terre et océan).
pub const DIFFUSIVITY_W_M2_K: f64 = 0.555;
/// Couche de mélange océanique, m équivalent eau.
pub const OCEAN_DEPTH_M: f64 = 70.0;
/// Capacité terrestre, m équivalent eau.
pub const LAND_DEPTH_M: f64 = 10.0;
/// Capacité de la glace, m équivalent eau.
pub const ICE_DEPTH_M: f64 = 10.0;
/// Gradient d'altitude appliqué à la température de surface des terres, K m⁻¹.
pub const LAPSE_RATE_K_PER_M: f64 = 6.5e-3;
/// Bornes de la température d'énergie, °C ; un dépassement est compté dans le rapport, un NaN est une erreur.
pub const TEMP_MIN_C: f64 = -270.0;
pub const TEMP_MAX_C: f64 = 1_700.0;
/// Pas de la relaxation vers l'équilibre, années (implicite : le pas ne dépend pas d'une stabilité explicite).
pub const EQUILIBRIUM_STEP_YEARS: f64 = 50.0;
/// Nombre maximal de pas de relaxation vers l'équilibre.
pub const EQUILIBRIUM_MAX_STEPS: u32 = 2_000;
/// Variation maximale de la température entre deux pas de relaxation pour déclarer l'équilibre, K.
pub const EQUILIBRIUM_TOLERANCE_K: f64 = 1e-4;
const EARTH_START_C: f64 = 15.0;
const KELVIN_OFFSET: f64 = 273.15;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Loi d'émission au sommet de l'atmosphère.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OlrLaw {
    /// Terre : `A + B·T − ΔF_CO₂`, T en °C, CO₂ pilotant le forçage.
    Earth,
    /// Atmosphère grise `σT⁴/(1 + 3τ/4)` ; τ = `tau_ref` à la concentration `co2_ref_ppm`, mis à l'échelle ensuite.
    Grey { tau_ref: f64, co2_ref_ppm: f64 },
}

/// Paramètres physiques, indépendants du maillage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClimateConfig {
    pub law: OlrLaw,
    /// Facteur multiplicatif de l'insolation (1 = S0 du corps ; 0,95 pour l'hystérésis).
    pub solar_scale: f64,
    /// Rétroaction glace-albédo active (vrai pour la Terre, faux pour les corps sans glace d'eau).
    pub ice_feedback: bool,
    pub diffusivity: f64,
}

impl ClimateConfig {
    pub fn earth() -> Self {
        Self {
            law: OlrLaw::Earth,
            solar_scale: 1.0,
            ice_feedback: true,
            diffusivity: DIFFUSIVITY_W_M2_K,
        }
    }

    /// Corps réel : Terre en loi terrestre, autres corps en atmosphère grise calibrée sur leur température observée.
    pub fn for_body(body: &Body) -> Result<Self, GreenhouseError> {
        if body.id == "earth" {
            return Ok(Self::earth());
        }
        let tau_ref = calibrate_tau(body)?;
        Ok(Self::grey(tau_ref, body.co2_fraction.unwrap_or(0.0) * 1e6))
    }

    /// Monde généré : τ = f(P)·g(CO₂) avec la pression totale et la concentration de départ.
    pub fn for_world(pressure_bar: f64, co2_ppm: f64) -> Self {
        let tau_ref = tau_for_world(pressure_bar, co2_ppm * 1e-6);
        Self::grey(tau_ref, co2_ppm)
    }

    fn grey(tau_ref: f64, co2_ref_ppm: f64) -> Self {
        Self {
            law: OlrLaw::Grey { tau_ref, co2_ref_ppm },
            solar_scale: 1.0,
            ice_feedback: false,
            diffusivity: DIFFUSIVITY_W_M2_K,
        }
    }
}

/// Surface figée à l'initialisation : produite par la genèse.
#[derive(Debug, Clone, Copy)]
pub struct Surface<'a> {
    pub elevation_m: &'a [f64],
    pub is_ocean: &'a [bool],
}

/// Entrées par pas, empruntées à d'autres domaines : CO₂ (matière) et albédo vivant optionnel.
#[derive(Debug, Clone, Copy)]
pub struct ClimateInputs<'a> {
    pub co2_ppm: f64,
    /// Albédo de surface libre par cellule (vivant) ; `None` = défaut par loi.
    pub surface_albedo: Option<&'a [f64]>,
}

/// Géométrie et paramètres figés d'un monde climatique.
pub(super) struct ClimateModel {
    pub(super) mesh: Mesh,
    pub(super) law: OlrLaw,
    pub(super) ice_feedback: bool,
    pub(super) diffusivity: f64,
    pub(super) surface_capacity: Vec<f64>,
    pub(super) ice_capacity: f64,
    pub(super) lapse_offset_k: Vec<f64>,
    pub(super) free_albedo: Vec<f64>,
    pub(super) insolation_monthly: Vec<Vec<f64>>,
    pub(super) insolation_annual: Vec<f64>,
    pub(super) year_s: f64,
}

/// Champs évolutifs. Températures et fractions par cellule ; flux diagnostiqués au dernier pas.
pub struct ClimateState {
    model: Arc<ClimateModel>,
    /// Température d'énergie (au niveau de la mer), °C : variable du bilan.
    pub energy_c: Vec<f64>,
    /// Température de surface, °C (gradient d'altitude appliqué sur les terres) : champ `temperature_c` du §12.
    pub temperature_c: Vec<f64>,
    /// Fraction de glace (champ `ice_fraction` du §12).
    pub ice_fraction: Vec<f64>,
    /// Insolation moyenne du pas, W m⁻² (champ `insolation_w_m2` du §12).
    pub insolation_w_m2: Vec<f64>,
    /// Flux absorbé et OLR évalués à T^n du dernier pas, W m⁻².
    pub absorbed_w_m2: Vec<f64>,
    pub olr_w_m2: Vec<f64>,
    /// Réservés au pas hydrologique : toujours nuls ici.
    pub precip_mm_yr: Vec<f64>,
    pub wind_u: Vec<f64>,
    pub wind_v: Vec<f64>,
    pub soil_moisture: Vec<f64>,
    /// Temps écoulé depuis l'initialisation, s.
    pub time_s: f64,
}

impl ClimateState {
    pub fn cell_count(&self) -> usize {
        self.model.mesh.cell_count()
    }

    pub fn area_sr(&self) -> &[f64] {
        self.model.mesh.area_sr()
    }

    pub fn sin_lat(&self) -> &[f64] {
        self.model.mesh.sin_lats()
    }
}

/// Équilibre atteint (ou non) par la relaxation stationnaire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquilibriumReport {
    pub iterations: u32,
    pub max_change_k: f64,
    pub converged: bool,
}

/// Bilan d'un appel à `step` : maxima sur les appels du solveur, compteurs de bornes et de plafonds, équilibre.
#[derive(Debug, Clone, PartialEq)]
pub struct ClimateReport {
    pub regime: Regime,
    /// Nombre de résolutions implicites lancées par cet appel.
    pub substeps: u32,
    pub max_step_change_k: f64,
    pub max_cg_iterations: u32,
    pub max_relative_residual: f64,
    /// Résidu d'énergie laissé par le CG avant la correction de mode constant, relatif à Σ|b|.
    pub max_energy_error: f64,
    pub clamped_cells: u32,
    /// Résolutions qui n'ont pas atteint le résidu visé avant le plafond d'itérations.
    pub cg_cap_hits: u32,
    pub equilibrium: Option<EquilibriumReport>,
}

impl ClimateReport {
    fn new(regime: Regime) -> Self {
        Self {
            regime,
            substeps: 0,
            max_step_change_k: 0.0,
            max_cg_iterations: 0,
            max_relative_residual: 0.0,
            max_energy_error: 0.0,
            clamped_cells: 0,
            cg_cap_hits: 0,
            equilibrium: None,
        }
    }

    fn absorb(&mut self, sub: &SubstepReport) {
        self.substeps += 1;
        self.max_step_change_k = self.max_step_change_k.max(sub.max_change_k);
        self.max_cg_iterations = self.max_cg_iterations.max(sub.cg_iterations);
        self.max_relative_residual = self.max_relative_residual.max(sub.relative_residual);
        self.max_energy_error = self.max_energy_error.max(sub.energy_error);
        self.clamped_cells += sub.clamped;
        self.cg_cap_hits += u32::from(sub.cap_reached);
    }
}

/// Construit le modèle et l'état initial d'un corps sur une grille.
pub fn init(
    grid: &Grid,
    body: &Body,
    surface: &Surface<'_>,
    config: &ClimateConfig,
) -> Result<ClimateState, ClimateError> {
    check_length("élévation", surface.elevation_m.len(), grid.cell_count())?;
    check_length("océan", surface.is_ocean.len(), grid.cell_count())?;
    let mesh = Mesh::new(grid)?;
    let table = build_table(body)?;
    let model = Arc::new(assemble(mesh, body, surface, config, &table));
    Ok(initial_state(model, body, config))
}

/// Avance l'état de `dt_years` années. `regime` doit être celui de `clock::regime(System::Climate, dt_years)`
/// (les régimes Fine et Annual sont interchangeables pour Δt ≤ 1 an). `tick` et `seed` sont inutilisés : le climat
/// est déterministe sans aléa.
pub fn step(
    state: &mut ClimateState,
    grid: &Grid,
    inputs: &ClimateInputs<'_>,
    dt_years: f64,
    regime: Regime,
    _tick: u64,
    _seed: u64,
) -> Result<ClimateReport, ClimateError> {
    check_length("grille", grid.cell_count(), state.cell_count())?;
    if !(dt_years.is_finite() && dt_years > 0.0) {
        return Err(ClimateError::InvalidDuration(dt_years));
    }
    check_regime(regime, regime_of(dt_years))?;
    let model = Arc::clone(&state.model);
    let run = Run {
        model: &model,
        co2_ppm: inputs.co2_ppm,
        free_albedo: inputs.surface_albedo.unwrap_or(&model.free_albedo),
    };
    match regime {
        Regime::Fine | Regime::Annual => seasonal_step(state, &run, dt_years, regime),
        Regime::Secular => annual_mean_step(state, &run, dt_years),
        Regime::Geological => equilibrium(state, &run),
        Regime::Frozen => Err(ClimateError::FrozenRegime),
    }
}

/// Paramètres communs d'un appel à `step`.
struct Run<'a> {
    model: &'a ClimateModel,
    co2_ppm: f64,
    free_albedo: &'a [f64],
}

fn regime_of(dt_years: f64) -> Regime {
    regime(System::Climate, dt_years)
}

fn check_regime(given: Regime, expected: Regime) -> Result<(), ClimateError> {
    let seasonal = |r: Regime| matches!(r, Regime::Fine | Regime::Annual);
    if given == expected || (seasonal(given) && seasonal(expected)) {
        Ok(())
    } else {
        Err(ClimateError::RegimeMismatch { given, expected })
    }
}

/// Pas saisonnier unique (Δt ≤ 1 an) avec l'insolation de la date du milieu du pas.
fn seasonal_step(
    state: &mut ClimateState,
    run: &Run<'_>,
    dt_years: f64,
    regime: Regime,
) -> Result<ClimateReport, ClimateError> {
    let mid = state.time_s + 0.5 * dt_years * run.model.year_s;
    let insolation = &run.model.insolation_monthly[month_index(run.model, mid)];
    let sub = advance(run, state, dt_years, insolation)?;
    let mut report = ClimateReport::new(regime);
    report.absorb(&sub);
    Ok(report)
}

/// Pas unique avec l'insolation moyenne annuelle (1 an < Δt ≤ 1 000 ans).
fn annual_mean_step(state: &mut ClimateState, run: &Run<'_>, dt_years: f64) -> Result<ClimateReport, ClimateError> {
    let sub = advance(run, state, dt_years, &run.model.insolation_annual)?;
    let mut report = ClimateReport::new(Regime::Secular);
    report.absorb(&sub);
    Ok(report)
}

/// Relaxation stationnaire : pas implicites à insolation annuelle jusqu'à convergence ou bornes atteintes.
fn equilibrium(state: &mut ClimateState, run: &Run<'_>) -> Result<ClimateReport, ClimateError> {
    let mut report = ClimateReport::new(Regime::Geological);
    let mut change = f64::INFINITY;
    let mut iterations = 0;
    while iterations < EQUILIBRIUM_MAX_STEPS && change > EQUILIBRIUM_TOLERANCE_K {
        let sub = advance(run, state, EQUILIBRIUM_STEP_YEARS, &run.model.insolation_annual)?;
        change = sub.max_change_k;
        report.absorb(&sub);
        iterations += 1;
    }
    report.equilibrium = Some(EquilibriumReport {
        iterations,
        max_change_k: change,
        converged: change <= EQUILIBRIUM_TOLERANCE_K,
    });
    Ok(report)
}

/// Un pas implicite avec l'insolation donnée ; met à jour le temps et l'insolation de sortie.
fn advance(
    run: &Run<'_>,
    state: &mut ClimateState,
    dt_years: f64,
    insolation: &[f64],
) -> Result<SubstepReport, ClimateError> {
    let dt_s = dt_years * run.model.year_s;
    let forcing = Forcing {
        co2_ppm: run.co2_ppm,
        dt_s,
        insolation,
        free_albedo: run.free_albedo,
    };
    let sub = ebm::substep(run.model, state, &forcing)?;
    state.time_s += dt_s;
    state.insolation_w_m2 = insolation.to_vec();
    Ok(sub)
}

/// Mois (0 = premier de l'année orbitale) contenant l'instant `time_s`.
fn month_index(model: &ClimateModel, time_s: f64) -> usize {
    let phase = (time_s / model.year_s).rem_euclid(1.0);
    ((phase * MONTHS as f64) as usize).min(MONTHS - 1)
}

fn check_length(field: &'static str, got: usize, expected: usize) -> Result<(), ClimateError> {
    if got == expected {
        Ok(())
    } else {
        Err(ClimateError::InputLength { field, expected, got })
    }
}

fn assemble(
    mesh: Mesh,
    body: &Body,
    surface: &Surface<'_>,
    config: &ClimateConfig,
    table: &InsolationTable,
) -> ClimateModel {
    let surface_capacity = surface
        .is_ocean
        .iter()
        .map(|&ocean| {
            let depth = if ocean { OCEAN_DEPTH_M } else { LAND_DEPTH_M };
            WATER_VOLUMETRIC_HEAT_CAPACITY * depth
        })
        .collect();
    let lapse_offset_k = surface
        .elevation_m
        .iter()
        .zip(surface.is_ocean)
        .map(|(&z, &ocean)| if ocean { 0.0 } else { LAPSE_RATE_K_PER_M * z.max(0.0) })
        .collect();
    let free_albedo = free_albedo_field(&mesh, config.law, body);
    let (insolation_monthly, insolation_annual) = insolation_fields(&mesh, table, config.solar_scale);
    ClimateModel {
        mesh,
        law: config.law,
        ice_feedback: config.ice_feedback,
        diffusivity: config.diffusivity,
        surface_capacity,
        ice_capacity: WATER_VOLUMETRIC_HEAT_CAPACITY * ICE_DEPTH_M,
        lapse_offset_k,
        free_albedo,
        insolation_monthly,
        insolation_annual,
        year_s: body.orbital_period_days * SECONDS_PER_DAY,
    }
}

/// Albédo libre par défaut : Budyko zonal pour la Terre, albédo de Bond uniforme pour les autres corps.
fn free_albedo_field(mesh: &Mesh, law: OlrLaw, body: &Body) -> Vec<f64> {
    match law {
        OlrLaw::Earth => mesh.sin_lats().iter().map(|&s| budyko_free_albedo(s)).collect(),
        OlrLaw::Grey { .. } => vec![body.bond_albedo; mesh.cell_count()],
    }
}

/// Insolation par mois (W m⁻²) et moyenne annuelle, par cellule, à partir de la table latitude × mois.
fn insolation_fields(mesh: &Mesh, table: &InsolationTable, scale: f64) -> (Vec<Vec<f64>>, Vec<f64>) {
    let monthly: Vec<Vec<f64>> = (0..MONTHS)
        .map(|month| {
            mesh.sin_lats()
                .iter()
                .map(|&s| scale * table.at(s.asin().to_degrees(), month))
                .collect()
        })
        .collect();
    let annual = (0..mesh.cell_count())
        .map(|i| monthly.iter().map(|m| m[i]).sum::<f64>() / MONTHS as f64)
        .collect();
    (monthly, annual)
}

fn initial_state(model: Arc<ClimateModel>, body: &Body, config: &ClimateConfig) -> ClimateState {
    let n = model.mesh.cell_count();
    let start_c = match config.law {
        OlrLaw::Earth => EARTH_START_C,
        OlrLaw::Grey { tau_ref, .. } => {
            let absorbed = body.solar_flux_wm2 * config.solar_scale * (1.0 - body.bond_albedo) / 4.0;
            grey_surface_temp_k(absorbed, tau_ref) - KELVIN_OFFSET
        }
    };
    let mut state = ClimateState {
        model,
        energy_c: vec![start_c; n],
        temperature_c: vec![0.0; n],
        ice_fraction: vec![0.0; n],
        insolation_w_m2: vec![0.0; n],
        absorbed_w_m2: vec![0.0; n],
        olr_w_m2: vec![0.0; n],
        precip_mm_yr: vec![0.0; n],
        wind_u: vec![0.0; n],
        wind_v: vec![0.0; n],
        soil_moisture: vec![0.0; n],
        time_s: 0.0,
    };
    let model = Arc::clone(&state.model);
    ebm::diagnose(&model, &mut state);
    state
}
