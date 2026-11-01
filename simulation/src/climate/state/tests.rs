use super::*;
use crate::climate::body::{body, Body};
use crate::climate::greenhouse::{co2_forcing, EARTH_OLR_B_W_M2_K};
use crate::clock::{regime, Regime, System};
use crate::shared::grid::Grid;
use crate::shared::units::R_EARTH_M;
use std::time::Instant;

/// Terre synthétique : continents en bandes de latitude, élévation nulle. La moyenne ne dépend pas de la
/// répartition, seule la capacité change.
struct World {
    grid: Grid,
    elevation: Vec<f64>,
    ocean: Vec<bool>,
}

fn world(level: u8) -> World {
    let grid = Grid::new(level, R_EARTH_M);
    let ocean = grid
        .centers()
        .iter()
        .map(|c| !(0.34..0.77).contains(&c[1].abs()))
        .collect();
    World {
        elevation: vec![0.0; grid.cell_count()],
        grid,
        ocean,
    }
}

fn surface(w: &World) -> Surface<'_> {
    Surface {
        elevation_m: &w.elevation,
        is_ocean: &w.ocean,
    }
}

fn start(w: &World, planet: &Body, config: &ClimateConfig) -> ClimateState {
    init(&w.grid, planet, &surface(w), config).expect("initialisation")
}

fn inputs(co2_ppm: f64) -> ClimateInputs<'static> {
    ClimateInputs {
        co2_ppm,
        surface_albedo: None,
    }
}

/// Un appel à `step` avec le régime que `clock` attribue à `dt_years`.
fn advance(state: &mut ClimateState, w: &World, co2_ppm: f64, dt_years: f64) -> ClimateReport {
    let chosen = regime(System::Climate, dt_years);
    step(state, &w.grid, &inputs(co2_ppm), dt_years, chosen, 0, 0).expect("pas climatique")
}

fn equilibrate(state: &mut ClimateState, w: &World, co2_ppm: f64) -> ClimateReport {
    advance(state, w, co2_ppm, 10_000.0)
}

fn earth_state(level: u8, config: &ClimateConfig) -> (World, ClimateState) {
    let w = world(level);
    let state = start(&w, &body("earth").expect("terre"), config);
    (w, state)
}

fn global_mean(state: &ClimateState, field: &[f64]) -> f64 {
    let total: f64 = state.area_sr().iter().zip(field).map(|(a, v)| a * v).sum();
    total / state.area_sr().iter().sum::<f64>()
}

fn zonal_mean(state: &ClimateState, field: &[f64], keep: impl Fn(f64) -> bool) -> f64 {
    let (sum, weight) = state
        .sin_lat()
        .iter()
        .zip(state.area_sr())
        .zip(field)
        .filter(|((s, _), _)| keep(s.abs()))
        .fold((0.0, 0.0), |(acc, w), ((_, a), v)| (acc + a * v, w + a));
    sum / weight
}

fn pool(threads: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("pool rayon")
}

#[test]
fn v06_earth_global_mean_is_287_2_kelvin() {
    let (w, mut state) = earth_state(5, &ClimateConfig::earth());
    equilibrate(&mut state, &w, 280.0);
    let mean_k = global_mean(&state, &state.temperature_c) + 273.15;
    assert!((mean_k - 287.2).abs() <= 1.5, "T̄ = {mean_k} K");
}

#[test]
fn v07_equator_to_pole_gradient_is_40_kelvin() {
    for level in [5_u8, 6] {
        let (w, mut state) = earth_state(level, &ClimateConfig::earth());
        equilibrate(&mut state, &w, 280.0);
        let equator = zonal_mean(&state, &state.temperature_c, |s| s < 0.17);
        let pole = zonal_mean(&state, &state.temperature_c, |s| s > 0.985);
        let gradient = equator - pole;
        assert!((gradient - 40.0).abs() <= 8.0, "L{level} : gradient {gradient} K");
    }
}

#[test]
fn v08_doubling_without_feedback_is_flux_over_b() {
    let config = ClimateConfig {
        ice_feedback: false,
        ..ClimateConfig::earth()
    };
    let (w, mut low) = earth_state(5, &config);
    equilibrate(&mut low, &w, 280.0);
    let (_, mut high) = earth_state(5, &config);
    equilibrate(&mut high, &w, 560.0);
    let delta = global_mean(&high, &high.temperature_c) - global_mean(&low, &low.temperature_c);
    let expected = co2_forcing(560.0) / EARTH_OLR_B_W_M2_K;
    assert!((delta - expected).abs() <= 0.05, "ΔT {delta} K, attendu {expected} K");
}

#[test]
fn v09_absorbed_and_emitted_balance_at_equilibrium() {
    let (w, mut state) = earth_state(5, &ClimateConfig::earth());
    equilibrate(&mut state, &w, 280.0);
    let imbalance = global_mean(&state, &state.absorbed_w_m2) - global_mean(&state, &state.olr_w_m2);
    assert!(imbalance.abs() < 0.5, "déséquilibre {imbalance} W/m²");
}

/// V-10 : six pas mensuels à forçage figé (même mois à chaque pas, pour isoler la stabilité numérique du cycle
/// saisonnier) : bornes respectées, convergence du solveur, pas d'oscillation de signe.
fn assert_monthly_stability(level: u8) {
    let (w, mut state) = earth_state(level, &ClimateConfig::earth());
    let mut previous = state.energy_c.clone();
    let mut last_change = vec![0.0_f64; previous.len()];
    for month in 0..6 {
        state.time_s = 0.0;
        let report = advance(&mut state, &w, 280.0, 1.0 / 12.0);
        assert!(
            report.max_step_change_k < 5.0,
            "L{level} mois {month} : ΔT {}",
            report.max_step_change_k
        );
        assert_eq!(
            report.cg_cap_hits, 0,
            "L{level} mois {month} : plafond CG atteint {report:?}"
        );
        let change: Vec<f64> = state.energy_c.iter().zip(&previous).map(|(n, p)| n - p).collect();
        // Le premier transitoire (champ initial uniforme, relaxé par le transport) peut inverser une fois la
        // tendance ; une oscillation numérique la renverserait sans fin. On exige donc une relaxation monotone à
        // partir du troisième pas.
        if month >= 2 {
            for (i, (&now, &before)) in change.iter().zip(&last_change).enumerate() {
                let flipped = now * before < 0.0 && now.abs() > 0.05 && before.abs() > 0.05;
                assert!(!flipped, "L{level} cellule {i} oscille au mois {month}");
            }
        }
        last_change = change;
        previous = state.energy_c.clone();
    }
}

#[test]
fn v10_monthly_step_is_stable_at_l6() {
    assert_monthly_stability(6);
}

#[test]
fn v10_monthly_step_is_stable_at_l7() {
    assert_monthly_stability(7);
}

#[test]
fn v16_hysteresis_frozen_and_warm_branches_coexist_at_095() {
    let config = ClimateConfig {
        solar_scale: 0.95,
        ..ClimateConfig::earth()
    };
    let (w, mut warm) = earth_state(4, &config);
    equilibrate(&mut warm, &w, 280.0);
    let (_, mut cold) = earth_state(4, &config);
    cold.energy_c.iter_mut().for_each(|t| *t = -50.0);
    equilibrate(&mut cold, &w, 280.0);
    let warm_mean = global_mean(&warm, &warm.temperature_c);
    let cold_mean = global_mean(&cold, &cold.temperature_c);
    assert!(warm_mean > 0.0, "branche tempérée à 0,95·S0 : {warm_mean} °C");
    assert!(cold_mean < -10.0, "branche gelée à 0,95·S0 : {cold_mean} °C");
}

/// V-17a : la boule de neige à S0 et CO₂ de référence reste gelée 500 ans simulés.
#[test]
fn v17a_snowball_stays_frozen_for_500_years_at_s0() {
    let (w, mut state) = earth_state(4, &ClimateConfig::earth());
    state.energy_c.iter_mut().for_each(|t| *t = -50.0);
    for decade in 1..=50 {
        advance(&mut state, &w, 280.0, 10.0);
        let ice = global_mean(&state, &state.ice_fraction);
        assert!(ice > 0.9, "fonte prématurée à {} ans : glace {ice}", decade * 10);
    }
}

/// V-17b : même boule de neige, CO₂ = 10 000 × 280 ppm (ΔF ≈ 49 W/m²) : glace sous 30 % avant 2 000 ans.
/// Seuil de déglaciation de l'EBM plus haut que la littérature (~0,1 bar) faute de rétroactions poussière/nuages.
#[test]
fn v17b_snowball_melts_within_2000_years_at_10000x_co2() {
    let co2 = 280.0 * 10_000.0;
    assert!((co2_forcing(co2) - 49.3).abs() < 0.5, "ΔF = {}", co2_forcing(co2));
    let (w, mut state) = earth_state(4, &ClimateConfig::earth());
    state.energy_c.iter_mut().for_each(|t| *t = -50.0);
    let mut melted_at = None;
    for decade in 1..=200 {
        advance(&mut state, &w, co2, 10.0);
        if global_mean(&state, &state.ice_fraction) < 0.30 {
            melted_at = Some(decade * 10);
            break;
        }
    }
    assert!(melted_at.is_some(), "glace encore au-dessus de 30 % à 2 000 ans");
}

#[test]
fn venus_full_model_surface_is_737_kelvin() {
    let w = world(5);
    let venus = body("venus").expect("vénus");
    let config = ClimateConfig::for_body(&venus).expect("calibration vénus");
    let mut state = start(&w, &venus, &config);
    equilibrate(&mut state, &w, 965_000.0);
    let mean_k = global_mean(&state, &state.temperature_c) + 273.15;
    assert!((mean_k - 737.0).abs() <= 10.0, "Vénus {mean_k} K");
}

#[test]
fn mars_full_model_surface_is_210_kelvin() {
    let w = world(5);
    let mars = body("mars").expect("mars");
    let config = ClimateConfig::for_body(&mars).expect("calibration mars");
    let mut state = start(&w, &mars, &config);
    equilibrate(&mut state, &w, 951_000.0);
    let mean_k = global_mean(&state, &state.temperature_c) + 273.15;
    assert!((mean_k - 210.0).abs() <= 5.0, "Mars {mean_k} K");
}

#[test]
fn regime_dispatch_follows_clock_thresholds() {
    let (w, mut state) = earth_state(3, &ClimateConfig::earth());
    let fine = advance(&mut state, &w, 280.0, 1.0 / 12.0);
    assert_eq!(fine.regime, Regime::Fine);
    assert_eq!(fine.substeps, 1);
    let annual = advance(&mut state, &w, 280.0, 0.5);
    assert_eq!(annual.regime, Regime::Annual);
    assert_eq!(annual.substeps, 1, "un pas saisonnier unique par Δt ≤ 1 an");
    let secular = advance(&mut state, &w, 280.0, 10.0);
    assert_eq!(secular.regime, Regime::Secular);
    assert_eq!(secular.substeps, 1);
    let geological = advance(&mut state, &w, 280.0, 5_000.0);
    assert_eq!(geological.regime, Regime::Geological);
    assert!(geological.equilibrium.is_some());
}

#[test]
fn regime_argument_must_match_the_clock() {
    let (w, mut state) = earth_state(3, &ClimateConfig::earth());
    let err = step(&mut state, &w.grid, &inputs(280.0), 10.0, Regime::Fine, 0, 0);
    assert!(matches!(err, Err(ClimateError::RegimeMismatch { .. })), "{err:?}");
}

#[test]
fn solver_energy_residual_before_correction_stays_small_at_l4() {
    let (w, mut state) = earth_state(4, &ClimateConfig::earth());
    for _ in 0..12 {
        let report = advance(&mut state, &w, 280.0, 1.0 / 12.0);
        assert!(report.max_energy_error < 1e-4, "{report:?}");
    }
}

#[test]
fn stepping_is_bitwise_deterministic() {
    let run = || {
        let (w, mut state) = earth_state(4, &ClimateConfig::earth());
        for _ in 0..3 {
            advance(&mut state, &w, 280.0, 1.0 / 12.0);
        }
        state.energy_c
    };
    let (a, b) = (run(), run());
    assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()));
}

#[test]
fn step_is_bitwise_identical_for_one_and_six_threads() {
    let run = |threads: usize| {
        pool(threads).install(|| {
            let (w, mut state) = earth_state(5, &ClimateConfig::earth());
            advance(&mut state, &w, 280.0, 1.0);
            advance(&mut state, &w, 280.0, 1.0 / 12.0);
            state.energy_c
        })
    };
    let (single, parallel) = (run(1), run(6));
    assert!(single.iter().zip(&parallel).all(|(x, y)| x.to_bits() == y.to_bits()));
}

#[test]
fn init_rejects_mismatched_surface_length() {
    let w = world(3);
    let short = vec![0.0; w.grid.cell_count() - 1];
    let bad = Surface {
        elevation_m: &short,
        is_ocean: &w.ocean,
    };
    let earth = body("earth").expect("terre");
    assert!(matches!(
        init(&w.grid, &earth, &bad, &ClimateConfig::earth()),
        Err(ClimateError::InputLength { .. })
    ));
}

#[test]
fn lapse_rate_lowers_reported_surface_temperature_only() {
    let grid = Grid::new(3, R_EARTH_M);
    let elevation = vec![1000.0; grid.cell_count()];
    let ocean = vec![false; grid.cell_count()];
    let earth = body("earth").expect("terre");
    let surface = Surface {
        elevation_m: &elevation,
        is_ocean: &ocean,
    };
    let state = init(&grid, &earth, &surface, &ClimateConfig::earth()).expect("init");
    let drop = state.energy_c[0] - state.temperature_c[0];
    assert!((drop - 6.5).abs() < 1e-9, "chute {drop} K");
}

/// Banc : un tick annuel, 6 threads, départ à froid (état initial). Cible : L6 < 60 ms, L7 < 250 ms.
fn annual_tick_ms(level: u8) -> (f64, ClimateReport) {
    pool(6).install(|| {
        let (w, mut state) = earth_state(level, &ClimateConfig::earth());
        let start = Instant::now();
        let report = advance(&mut state, &w, 280.0, 1.0);
        (start.elapsed().as_secs_f64() * 1e3, report)
    })
}

#[test]
#[ignore = "banc release : un tick annuel L6 < 100 ms sur 6 cœurs"]
fn bench_annual_tick_l6_under_100_ms() {
    let (ms, report) = annual_tick_ms(6);
    assert!(ms < 100.0, "tick annuel L6 : {ms} ms, {report:?}");
}

/// Informatif seulement : affiche le temps d'un tick annuel L7 sur 6 cœurs, sans seuil.
#[test]
#[ignore = "banc release informatif : tick annuel L7 (affiché, sans seuil)"]
fn bench_annual_tick_l7_informative() {
    let (ms, report) = annual_tick_ms(7);
    println!("BANC L7 informatif : {ms:.1} ms, {report:?}");
}
