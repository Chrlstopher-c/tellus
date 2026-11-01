//! Cycle de l'eau (REGLES §5, climat.md §5-6) : saturation de Bolton, évaporation Clausius–Clapeyron, transport de
//! vapeur stationnaire par saison (hydro/transport.rs), précipitation par condensation et orographique, humidité du
//! sol en seau. Diagnostic, jamais à chaque tick : recalculé tous les `DIAGNOSTIC_INTERVAL_YEARS` simulés ou si
//! |ΔT̄| > `DIAGNOSTIC_DELTA_K`.

mod transport;
#[cfg(test)]
mod tests;

use super::body::Body;
use super::ice::ice_fraction;
use super::insolation::MONTHS;
use super::mesh::Mesh;
use super::orography;
use super::state::{ClimateModel, ClimateState, Surface};
use super::wind::{self, Circulation};
use crate::shared::units::SECONDS_PER_DAY;
use std::f64::consts::TAU;
use transport::{directed_flux, SolveStats, Steady};

/// Intervalle de recalcul diagnostic, années simulées.
pub const DIAGNOSTIC_INTERVAL_YEARS: f64 = 10.0;
/// Dérive de la température moyenne qui déclenche un recalcul, K.
pub const DIAGNOSTIC_DELTA_K: f64 = 0.5;
/// Saisons représentatives (mois de départ de chacune, trois mois chacune).
const SEASONS: usize = 4;
const SEASON_MONTHS: [usize; SEASONS] = [0, 3, 6, 9];
/// Pression de référence pour la saturation, hPa.
const PRESSURE_HPA: f64 = 1_000.0;
/// Masse de la couche d'air humide : `W = q·M`, kg m⁻² [décision].
const MOIST_LAYER_KG_M2: f64 = 2_000.0;
/// Masse volumique de l'air de surface, kg m⁻³.
const AIR_DENSITY_KG_M3: f64 = 1.2;
/// Coefficient de transfert turbulent de l'évaporation, sans dimension, calé sur P̄ ≈ 1 000 mm an⁻¹ (V-22) [décision].
const EVAP_COEFF: f64 = 1.5e-3;
/// Condensation au-delà de cette humidité relative de la couche limite : la convection évacue la vapeur que la
/// seule convergence ne peut pas épuiser. Calée sur P̄ ≈ 1 000 mm an⁻¹ (V-22) ; HR océanique observée ≈ 0,8 [décision].
const RELATIVE_HUMIDITY_CAP: f64 = 0.7;
/// Rafales sous-maille ajoutées à la vitesse moyenne (|U| = √(u² + v² + G²)), m s⁻¹ [décision, gustiness].
const WIND_FLOOR_M_S: f64 = 4.0;
/// Temps de conversion de la vapeur saturée en pluie, s (1,5 h).
const RAIN_TIME_S: f64 = 5_400.0;
/// Capacité du seau d'humidité du sol, mm (kg m⁻²).
const BUCKET_CAPACITY_MM: f64 = 150.0;
/// Passes du seau sur le cycle saisonnier avant la sortie.
const BUCKET_CYCLES: u32 = 2;
/// Départ à chaud : fraction de la saturation de la colonne.
const INITIAL_MOISTURE_RATIO: f64 = 0.8;

/// Rapport diagnostic : moyennes globales et état du solveur.
#[derive(Debug, Clone, PartialEq)]
pub struct HydroReport {
    pub mean_precip_mm_yr: f64,
    pub mean_evap_mm_yr: f64,
    pub krylov_iterations: u32,
    pub unconverged_seasons: u32,
    pub clamped_cells: u32,
}

/// Pression de vapeur saturante (Bolton 1980), hPa ; T en °C, précision 0,1 % sur −30 à 35 °C.
pub fn saturation_vapour_hpa(temp_c: f64) -> f64 {
    6.112 * (17.67 * temp_c / (temp_c + 243.5)).exp()
}

/// Rapport de mélange saturant, kg kg⁻¹, à `PRESSURE_HPA`.
pub fn saturation_humidity(temp_c: f64) -> f64 {
    0.622 * saturation_vapour_hpa(temp_c) / PRESSURE_HPA
}

/// Dérivée logarithmique `d(ln e_s)/dT`, K⁻¹ (Clausius–Clapeyron, forme de Bolton).
pub fn clausius_clapeyron_slope(temp_c: f64) -> f64 {
    17.67 * 243.5 / (temp_c + 243.5).powi(2)
}

/// Données fixes du cycle de l'eau d'un monde : océans, orographie, circulation par saison.
pub(super) struct HydroModel {
    ocean: Vec<bool>,
    gradients: Vec<[f64; 3]>,
    circulations: [Circulation; SEASONS],
    pub(super) year_s: f64,
}

impl HydroModel {
    pub(super) fn new(mesh: &Mesh, body: &Body, surface: &Surface<'_>, insolation: &[Vec<f64>]) -> Self {
        let omega = TAU / (body.sidereal_rotation_hours * 3600.0);
        let omega_rel = wind::EARTH_SIDEREAL_DAY_HOURS / body.sidereal_rotation_hours;
        let radius = mesh.radius_m();
        let circulations = SEASON_MONTHS
            .map(|month| Circulation::new(itcz_latitude(mesh, insolation, month), omega, radius, omega_rel));
        Self {
            ocean: surface.is_ocean.to_vec(),
            gradients: orography::gradient(mesh, surface.elevation_m),
            circulations,
            year_s: body.orbital_period_days * SECONDS_PER_DAY,
        }
    }
}

/// Latitude de la ZCIT pour un mois : cellule où l'insolation du mois précédent est maximale (retard d'un mois).
fn itcz_latitude(mesh: &Mesh, insolation: &[Vec<f64>], month: usize) -> f64 {
    let lagged = &insolation[(month + MONTHS - 1) % MONTHS];
    let best = lagged
        .iter()
        .enumerate()
        .fold((0, f64::NEG_INFINITY), |acc, (cell, &q)| if q > acc.1 { (cell, q) } else { acc })
        .0;
    mesh.sin_lat(best).asin()
}

/// Champs d'une saison : vents, flux de précipitation et d'évaporation par cellule, kg m⁻² s⁻¹.
struct Season {
    zonal: Vec<f64>,
    meridional: Vec<f64>,
    precip: Vec<f64>,
    evap: Vec<f64>,
    stats: SolveStats,
    clamped: u32,
}

/// Coefficients par cellule d'une saison : `c = β ρ C_E |U|` (kg m⁻² s⁻¹), `q_sat`, `W_sat = q_sat·M`.
struct Terms {
    coeff: Vec<f64>,
    qsat: Vec<f64>,
    wsat: Vec<f64>,
}

/// Calcule le cycle de l'eau et écrit les champs de diagnostic dans `state`.
pub(super) fn diagnose(model: &ClimateModel, hydro: &HydroModel, state: &mut ClimateState) -> HydroReport {
    let mesh = &model.mesh;
    let temp = state.temperature_c.clone();
    if !hydro.ocean.iter().any(|&sea| sea) {
        // Sans océan il n'y a aucune source de vapeur : le transport conservatif garderait l'eau initiale.
        let zeros = vec![0.0; mesh.cell_count()];
        return publish(model, hydro, state, &Accumulator::new(mesh.cell_count()), (&temp, &zeros));
    }
    let mut bucket: Vec<f64> = state.soil_moisture.iter().map(|f| f.clamp(0.0, 1.0) * BUCKET_CAPACITY_MM).collect();
    let mut warm: Vec<f64> = (0..mesh.cell_count())
        .map(|i| INITIAL_MOISTURE_RATIO * saturation_humidity(temp[i]) * MOIST_LAYER_KG_M2)
        .collect();
    let mut acc = Accumulator::new(mesh.cell_count());
    let dt = hydro.year_s / SEASONS as f64;
    for cycle in 0..BUCKET_CYCLES {
        for season in 0..SEASONS {
            let beta = moisture_factor(&hydro.ocean, &bucket);
            let out = run_season(model, hydro, season, &temp, &beta, &mut warm);
            update_bucket(&mut bucket, &hydro.ocean, &out, dt);
            if cycle + 1 == BUCKET_CYCLES {
                acc.add(&out);
            }
        }
    }
    publish(model, hydro, state, &acc, (&temp, &bucket))
}

/// Recalcul si l'intervalle de temps simulé ou la dérive de T̄ l'exige ; `None` sinon (chemin critique intact).
pub(super) fn refresh_if_due(model: &ClimateModel, state: &mut ClimateState) -> Option<HydroReport> {
    let hydro = model.hydro.as_ref()?;
    let mean = area_mean(&model.mesh, &state.temperature_c);
    let elapsed_years = (state.time_s - state.hydro_time_s) / hydro.year_s;
    if elapsed_years < DIAGNOSTIC_INTERVAL_YEARS && (mean - state.hydro_mean_c).abs() <= DIAGNOSTIC_DELTA_K {
        return None;
    }
    let report = diagnose(model, hydro, state);
    state.hydro_time_s = state.time_s;
    state.hydro_mean_c = mean;
    Some(report)
}

/// Une saison stationnaire : vents, coefficients, résolution du transport, précipitations et évaporation.
fn run_season(
    model: &ClimateModel,
    hydro: &HydroModel,
    season: usize,
    temp_c: &[f64],
    beta: &[f64],
    warm: &mut [f64],
) -> Season {
    let mesh = &model.mesh;
    let circulation = &hydro.circulations[season];
    let (zonal, meridional) = surface_winds(mesh, circulation);
    let winds = wind_vectors(mesh, &zonal, &meridional);
    let flux = directed_flux(mesh, &winds);
    let lift = orography::uplift_rate(&hydro.gradients, &winds, mesh.radius_m());
    let speed: Vec<f64> = zonal
        .iter()
        .zip(&meridional)
        .map(|(u, v)| (u * u + v * v + WIND_FLOOR_M_S * WIND_FLOOR_M_S).sqrt())
        .collect();
    let terms = cell_terms(temp_c, beta, &speed);
    let (lin, source) = linear_terms(mesh, &terms, &lift);
    let steady = Steady::new(mesh, &flux, lin, source, terms.wsat.clone(), RAIN_TIME_S);
    let stats = steady.solve(warm);
    let clamped = clamp_non_negative(warm);
    let (precip, evap) = fluxes(&terms, &lift, warm);
    Season { zonal, meridional, precip, evap, stats, clamped }
}

/// Vents de surface par cellule, m s⁻¹ (latitude de chaque cellule depuis le sinus stocké).
fn surface_winds(mesh: &Mesh, circulation: &Circulation) -> (Vec<f64>, Vec<f64>) {
    (0..mesh.cell_count())
        .map(|cell| wind::surface_wind(circulation, mesh.sin_lat(cell).asin()))
        .unzip()
}

/// Vecteurs vent 3D tangents : `u·e_est + v·e_nord`.
fn wind_vectors(mesh: &Mesh, zonal: &[f64], meridional: &[f64]) -> Vec<[f64; 3]> {
    let (east, north) = (mesh.east(), mesh.north());
    (0..mesh.cell_count())
        .map(|i| std::array::from_fn(|a| zonal[i] * east[i][a] + meridional[i] * north[i][a]))
        .collect()
}

/// Coefficients : évaporation `c = β ρ C_E |U|`, saturation `q_sat(T)`, seuil `W_c = HR_c·q_sat·M`.
fn cell_terms(temp_c: &[f64], beta: &[f64], speed: &[f64]) -> Terms {
    let coeff: Vec<f64> = (0..temp_c.len())
        .map(|i| beta[i] * AIR_DENSITY_KG_M3 * EVAP_COEFF * speed[i])
        .collect();
    let qsat: Vec<f64> = temp_c.iter().map(|&t| saturation_humidity(t)).collect();
    let wsat = qsat
        .iter()
        .map(|q| RELATIVE_HUMIDITY_CAP * q * MOIST_LAYER_KG_M2)
        .collect();
    Terms { coeff, qsat, wsat }
}

/// Termes linéaires du bilan en kg s⁻¹ : `A(s + λ)` (diagonale) et `A·c·q_sat` (source), avec `s = c/M`.
fn linear_terms(mesh: &Mesh, terms: &Terms, lift: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let radius = mesh.radius_m();
    let mut lin = Vec::with_capacity(terms.coeff.len());
    let mut source = Vec::with_capacity(terms.coeff.len());
    for (i, &c) in terms.coeff.iter().enumerate() {
        let area = mesh.area_sr()[i] * radius * radius;
        lin.push(area * (c / MOIST_LAYER_KG_M2 + lift[i]));
        source.push(area * c * terms.qsat[i]);
    }
    (lin, source)
}

/// Précipitations `P = max(0, W − W_sat)/τ + λ·W` et évaporation `E = c (q_sat − W/M)`, kg m⁻² s⁻¹.
fn fluxes(terms: &Terms, lift: &[f64], w: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let precip = (0..w.len())
        .map(|i| (w[i] - terms.wsat[i]).max(0.0) / RAIN_TIME_S + lift[i] * w[i])
        .collect();
    let evap = (0..w.len())
        .map(|i| terms.coeff[i] * (terms.qsat[i] - w[i] / MOIST_LAYER_KG_M2))
        .collect();
    (precip, evap)
}

fn clamp_non_negative(w: &mut [f64]) -> u32 {
    let mut clamped = 0;
    for value in w.iter_mut().filter(|v| **v < 0.0) {
        *value = 0.0;
        clamped += 1;
    }
    clamped
}

/// Facteur d'évaporation par cellule : 1 sur les océans, `S/S_max` sur les terres.
fn moisture_factor(ocean: &[bool], bucket: &[f64]) -> Vec<f64> {
    ocean
        .iter()
        .zip(bucket)
        .map(|(&sea, &s)| if sea { 1.0 } else { (s / BUCKET_CAPACITY_MM).clamp(0.0, 1.0) })
        .collect()
}

/// Seau d'humidité : `S ← clamp(S + (P − E)·Δt, 0, S_max)` sur les terres.
fn update_bucket(bucket: &mut [f64], ocean: &[bool], season: &Season, dt_s: f64) {
    for (i, slot) in bucket.iter_mut().enumerate() {
        if !ocean[i] {
            *slot = (*slot + (season.precip[i] - season.evap[i]) * dt_s).clamp(0.0, BUCKET_CAPACITY_MM);
        }
    }
}

/// Sommes sur les saisons de la dernière passe du seau et compteurs du solveur.
struct Accumulator {
    zonal: Vec<f64>,
    meridional: Vec<f64>,
    precip: Vec<f64>,
    evap: Vec<f64>,
    iterations: u32,
    unconverged: u32,
    clamped: u32,
}

impl Accumulator {
    fn new(cells: usize) -> Self {
        Self {
            zonal: vec![0.0; cells],
            meridional: vec![0.0; cells],
            precip: vec![0.0; cells],
            evap: vec![0.0; cells],
            iterations: 0,
            unconverged: 0,
            clamped: 0,
        }
    }

    fn add(&mut self, season: &Season) {
        accumulate(&mut self.zonal, &season.zonal);
        accumulate(&mut self.meridional, &season.meridional);
        accumulate(&mut self.precip, &season.precip);
        accumulate(&mut self.evap, &season.evap);
        self.iterations += season.stats.iterations;
        self.unconverged += u32::from(!season.stats.converged);
        self.clamped += season.clamped;
    }
}

fn accumulate(sum: &mut [f64], values: &[f64]) {
    for (s, v) in sum.iter_mut().zip(values) {
        *s += v;
    }
}

/// Écrit les champs de diagnostic dans l'état et renvoie les moyennes globales.
fn publish(
    model: &ClimateModel,
    hydro: &HydroModel,
    state: &mut ClimateState,
    acc: &Accumulator,
    fields: (&[f64], &[f64]),
) -> HydroReport {
    let (temp, bucket) = fields;
    let scale = 1.0 / SEASONS as f64;
    let mean_precip = area_mean(&model.mesh, &scaled(&acc.precip, scale * hydro.year_s));
    let mean_evap = area_mean(&model.mesh, &scaled(&acc.evap, scale * hydro.year_s));
    state.precip_mm_yr = scaled(&acc.precip, scale * hydro.year_s);
    state.wind_u = scaled(&acc.zonal, scale);
    state.wind_v = scaled(&acc.meridional, scale);
    state.soil_moisture = bucket
        .iter()
        .zip(&hydro.ocean)
        .map(|(s, &sea)| if sea { 0.0 } else { s / BUCKET_CAPACITY_MM })
        .collect();
    state.sea_ice_fraction = temp
        .iter()
        .zip(&hydro.ocean)
        .map(|(&t, &sea)| if sea { ice_fraction(t) } else { 0.0 })
        .collect();
    HydroReport {
        mean_precip_mm_yr: mean_precip,
        mean_evap_mm_yr: mean_evap,
        krylov_iterations: acc.iterations,
        unconverged_seasons: acc.unconverged,
        clamped_cells: acc.clamped,
    }
}

fn scaled(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter().map(|v| v * factor).collect()
}

/// Moyenne pondérée par l'aire de la sphère.
fn area_mean(mesh: &Mesh, values: &[f64]) -> f64 {
    let areas = mesh.area_sr();
    let total: f64 = areas.iter().sum();
    areas.iter().zip(values).map(|(a, v)| a * v).sum::<f64>() / total
}
