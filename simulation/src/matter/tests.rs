//! Tests du domaine matière (REGLES § 6, nutriments § Tests) : conservation par élément, positivité, ordres de
//! grandeur, thermostat, oxygène, déterminisme. Les tests longs (10 000 ans, thermostat, bancs) sont `#[ignore]` et
//! se lancent en release : `cargo test --release -- --ignored matter`.

use super::carbon;
use super::forcing::Forcing;
use super::nitrogen;
use super::params::{
    BURIAL_FRACTION, CN_VEG, CP_VEG, HUMIFICATION, K_FAST_PER_YR, K_SLOW_PER_YR, K_VEG_PER_YR, ORGANIC_RECYCLE,
    PPM_REF, SILICATE_REF_GTC_YR, VOLCANIC_GTC_YR, WEATHER_CO2_EXP,
};
use super::phosphorus;
use super::state::Element;
use super::*;
use crate::clock::{regime, System};
use crate::shared::grid::Grid;
use crate::shared::units::GRAMS_PER_GT;
use std::time::Instant;

const EARTH_RADIUS_M: f64 = 6.371e6;
const NPP_LAND_GC_M2_YR: f64 = 400.0;
const NPP_OCEAN_GC_M2_YR: f64 = 150.0;
const DRIFT_TOLERANCE: f64 = 1e-9;
const INVENTORY_PPM: f64 = 4_836.0;
const LAND_N_G_M2: f64 = 12.0;
const LAND_P_G_M2: f64 = 1.5;
const DEEP_C_GTC: f64 = 7_500.0;
const DEEP_N_GTN: f64 = 600.0;
const DEEP_P_GTP: f64 = 85.0;

/// Champ climatique synthétique sur une grille géodésique : T selon la latitude, P en bandes, continents en bandes.
struct Field {
    grid: Grid,
    areas_m2: Vec<f64>,
    temperature_c: Vec<f32>,
    precip_mm_yr: Vec<f32>,
    insolation_w_m2: Vec<f32>,
    ice_fraction: Vec<f32>,
    is_ocean: Vec<bool>,
    elevation_m: Vec<f32>,
    npp_gc_m2_yr: Vec<f32>,
    biomass_gc_m2: Vec<f32>,
}

/// Valeurs d'une cellule synthétique, dans l'ordre des tranches de `Field`.
struct CellValues {
    temperature_c: f32,
    precip_mm_yr: f32,
    insolation_w_m2: f32,
    ice_fraction: f32,
    is_ocean: bool,
    elevation_m: f32,
    npp_gc_m2_yr: f32,
    biomass_gc_m2: f32,
}

fn cell_values(lat: f64, lon: f64, npp_land: f64, npp_ocean: f64) -> CellValues {
    let land = (2.0 * lon).cos() * lat.cos() + 0.3 * (2.0 * lat).sin() > 0.45;
    let ice = if lat.abs() > 1.3 { 1.0 } else { 0.0 };
    let sin2 = lat.sin() * lat.sin();
    let npp = match (ice > 0.0, land) {
        (true, _) => 0.0,
        (false, true) => npp_land,
        (false, false) => npp_ocean,
    };
    CellValues {
        temperature_c: (27.0 - 40.0 * sin2) as f32,
        precip_mm_yr: (1000.0 + 500.0 * (3.0 * lat).cos()) as f32,
        insolation_w_m2: (340.0 * (1.0 - 0.45 * sin2)) as f32,
        ice_fraction: ice as f32,
        is_ocean: !land,
        elevation_m: if land { 400.0 } else { 0.0 },
        npp_gc_m2_yr: npp as f32,
        biomass_gc_m2: if land { 5_000.0 } else { 0.0 },
    }
}

impl Field {
    fn synthetic(level: u8, npp_land: f64, npp_ocean: f64) -> Self {
        let grid = Grid::new(level, EARTH_RADIUS_M);
        let mut field = Self {
            areas_m2: grid.areas_m2().to_vec(),
            grid,
            temperature_c: Vec::new(),
            precip_mm_yr: Vec::new(),
            insolation_w_m2: Vec::new(),
            ice_fraction: Vec::new(),
            is_ocean: Vec::new(),
            elevation_m: Vec::new(),
            npp_gc_m2_yr: Vec::new(),
            biomass_gc_m2: Vec::new(),
        };
        for c in field.grid.centers() {
            let norm = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            let v = cell_values((c[2] / norm).asin(), c[1].atan2(c[0]), npp_land, npp_ocean);
            field.temperature_c.push(v.temperature_c);
            field.precip_mm_yr.push(v.precip_mm_yr);
            field.insolation_w_m2.push(v.insolation_w_m2);
            field.ice_fraction.push(v.ice_fraction);
            field.is_ocean.push(v.is_ocean);
            field.elevation_m.push(v.elevation_m);
            field.npp_gc_m2_yr.push(v.npp_gc_m2_yr);
            field.biomass_gc_m2.push(v.biomass_gc_m2);
        }
        field
    }

    fn inputs(&self) -> MatterInputs<'_> {
        MatterInputs {
            temperature_c: &self.temperature_c,
            precip_mm_yr: &self.precip_mm_yr,
            insolation_w_m2: &self.insolation_w_m2,
            ice_fraction: &self.ice_fraction,
            is_ocean: &self.is_ocean,
            elevation_m: &self.elevation_m,
            npp_realized_gc_m2_yr: &self.npp_gc_m2_yr,
            biomass_gc_m2: &self.biomass_gc_m2,
        }
    }

    fn land_cells(&self) -> Vec<usize> {
        (0..self.is_ocean.len()).filter(|&i| !self.is_ocean[i]).collect()
    }
}

/// Un tick au régime correspondant à `dt_years`, sur le champ `field`.
fn run(state: &mut MatterState, field: &Field, dt_years: f64) -> Result<MatterReport, MatterError> {
    let tick_regime = regime(System::Matter, dt_years);
    step(state, &field.grid, &field.inputs(), dt_years, tick_regime, 0, 0)
}

/// Fixe la somme d'un élément à `total0` : le stock `sink` absorbe la différence avec les autres réservoirs.
fn rebalance(element: &mut Element, sink: usize, total0: f64) {
    let others = element.total() - element.get(sink);
    element.stocks[sink] = total0 - others;
    element.reopen();
}

/// État proche de l'équilibre d'une Terre : végétation, sols, réservoirs profonds et N/P à leurs valeurs de régime.
fn earth_like(field: &Field) -> MatterState {
    let config = MatterConfig {
        co2_ppm: INVENTORY_PPM,
        volcanic_factor: 1.0,
    };
    let mut state = init(&config, &field.grid, &field.inputs()).expect("init");
    let (c_total, n_total, p_total) = (state.carbon.total(), state.nitrogen.total(), state.phosphorus.total());
    let (c, n, p) = (state.carbon.layout, state.nitrogen.layout, state.phosphorus.layout);
    for i in field.land_cells() {
        let area = field.areas_m2[i];
        let npp_gtc = f64::from(field.npp_gc_m2_yr[i]) * area / GRAMS_PER_GT;
        let veg = npp_gtc / K_VEG_PER_YR;
        let fast = npp_gtc / K_FAST_PER_YR;
        let soil = HUMIFICATION * npp_gtc / K_SLOW_PER_YR + fast;
        state.carbon.stocks[c.at(carbon::VEG, i)] = veg;
        state.carbon.stocks[c.at(carbon::FAST, i)] = fast;
        state.carbon.stocks[c.at(carbon::SLOW, i)] = (soil - fast).max(0.0);
        state.nitrogen.stocks[n.at(nitrogen::VEG, i)] = veg / CN_VEG;
        state.nitrogen.stocks[n.at(nitrogen::SOIL, i)] = soil / CN_VEG;
        state.nitrogen.stocks[n.at(nitrogen::MIN, i)] = LAND_N_G_M2 * area / GRAMS_PER_GT;
        state.phosphorus.stocks[p.at(phosphorus::VEG, i)] = veg / CP_VEG;
        state.phosphorus.stocks[p.at(phosphorus::SOIL, i)] = soil / CP_VEG;
        state.phosphorus.stocks[p.at(phosphorus::LAB, i)] = LAND_P_G_M2 * area / GRAMS_PER_GT;
    }
    state.carbon.stocks[c.glob(carbon::DEEP)] = DEEP_C_GTC;
    state.nitrogen.stocks[n.glob(nitrogen::DEEP)] = DEEP_N_GTN;
    state.phosphorus.stocks[p.glob(phosphorus::DEEP)] = DEEP_P_GTP;
    rebalance(&mut state.carbon, c.glob(carbon::ATM), c_total);
    rebalance(&mut state.nitrogen, n.glob(nitrogen::N2), n_total);
    rebalance(&mut state.phosphorus, p.glob(phosphorus::APATITE), p_total);
    state
}

fn assert_clean(report: &MatterReport) {
    assert!(
        report.drift.max() <= DRIFT_TOLERANCE,
        "dérive par élément : {:?}",
        report.drift
    );
    assert_eq!(report.anomalies, 0, "anomalies numériques");
}

fn assert_non_negative(state: &MatterState) {
    for element in state.elements() {
        assert!(
            element.stocks.iter().all(|&s| s >= 0.0 && s.is_finite()),
            "stock négatif ou non fini"
        );
    }
}

#[test]
fn init_rejects_mismatched_inputs() {
    let field = Field::synthetic(3, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut inputs = field.inputs();
    inputs.temperature_c = &field.temperature_c[..10];
    let result = init(&MatterConfig::default(), &field.grid, &inputs);
    assert!(matches!(
        result,
        Err(MatterError::LengthMismatch {
            field: "temperature_c",
            ..
        })
    ));
}

#[test]
fn init_rejects_non_finite_forcing() {
    let mut field = Field::synthetic(3, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    field.precip_mm_yr[3] = f32::NAN;
    let result = init(&MatterConfig::default(), &field.grid, &field.inputs());
    assert!(matches!(
        result,
        Err(MatterError::NonFinite {
            field: "precip_mm_yr",
            index: 3
        })
    ));
}

#[test]
fn step_rejects_invalid_time_step_and_changed_mask() {
    let field = Field::synthetic(3, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = init(&MatterConfig::default(), &field.grid, &field.inputs()).expect("init");
    for dt in [0.0, -1.0, f64::NAN] {
        let result = run(&mut state, &field, dt);
        assert!(matches!(result, Err(MatterError::InvalidTimeStep(_))), "dt = {dt}");
    }
    let mut flipped = field.is_ocean.clone();
    flipped[0] = !flipped[0];
    let mut inputs = field.inputs();
    inputs.is_ocean = &flipped;
    let tick_regime = regime(System::Matter, 1.0);
    assert!(matches!(
        step(&mut state, &field.grid, &inputs, 1.0, tick_regime, 0, 0),
        Err(MatterError::MaskChanged)
    ));
}

#[test]
fn annual_ticks_conserve_every_element_and_stay_positive() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    for _ in 0..30 {
        let report = run(&mut state, &field, 1.0).expect("tick annuel");
        assert_clean(&report);
        assert_eq!(
            report.shortfalls,
            0,
            "transfert limité au stock source : C{} N{} P{} W{} O{}",
            state.carbon.shortfalls,
            state.nitrogen.shortfalls,
            state.phosphorus.shortfalls,
            state.water.shortfalls,
            state.oxygen.shortfalls
        );
        assert_non_negative(&state);
    }
}

#[test]
fn megayear_tick_conserves_every_element_and_stays_positive() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    let report = run(&mut state, &field, 1.0e6).expect("tick d'un million d'années");
    assert_clean(&report);
    assert_non_negative(&state);
}

#[test]
fn young_planet_without_life_keeps_oxygen_and_npp_at_zero() {
    let field = Field::synthetic(5, 0.0, 0.0);
    let config = MatterConfig {
        co2_ppm: 2_000.0,
        volcanic_factor: 1.0,
    };
    let mut state = init(&config, &field.grid, &field.inputs()).expect("init");
    for _ in 0..100 {
        let report = run(&mut state, &field, 10.0).expect("tick");
        assert_clean(&report);
        assert_eq!(state.o2_tmol(), 0.0);
        assert_eq!(report.npp_land_gtc_yr + report.npp_ocean_gtc_yr, 0.0);
    }
}

#[test]
fn organic_burial_raises_oxygen_every_tick() {
    let field = Field::synthetic(5, 0.0, NPP_OCEAN_GC_M2_YR);
    let config = MatterConfig {
        co2_ppm: 280.0,
        volcanic_factor: 1.0,
    };
    let mut state = init(&config, &field.grid, &field.inputs()).expect("init");
    let mut previous = state.o2_tmol();
    for _ in 0..60 {
        let report = run(&mut state, &field, 10.0).expect("tick");
        assert_clean(&report);
        assert!(
            state.o2_tmol() > previous,
            "O2 doit monter avec l'enfouissement organique : O2 {} précédent {previous}, NPP {} GtC/an, DIN {} deep {} DIP {} deepP {}",
            state.o2_tmol(),
            report.npp_ocean_gtc_yr,
            state.nitrogen.total(),
            state.nitrogen.get(state.nitrogen.layout.glob(nitrogen::DEEP)),
            state.phosphorus.total(),
            state.phosphorus.get(state.phosphorus.layout.glob(phosphorus::DEEP))
        );
        previous = state.o2_tmol();
    }
}

#[test]
fn liebig_caps_npp_when_nitrogen_supply_is_only_fixation() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, 0.0);
    let mut state = earth_like(&field);
    let land = field.land_cells();
    let fed = Forcing::compute(&state, &field.inputs());
    let fed_max = land.iter().map(|&i| fed.npp_cap[i]).fold(0.0, f64::max);
    let n = state.nitrogen.layout;
    for &i in &land {
        state.nitrogen.stocks[n.at(nitrogen::MIN, i)] = 0.0;
        state.nitrogen.stocks[n.at(nitrogen::SOIL, i)] = 0.0;
    }
    state.nitrogen.reopen();
    let starved = Forcing::compute(&state, &field.inputs());
    let starved_max = land.iter().map(|&i| starved.npp_cap[i]).fold(0.0, f64::max);
    assert!(starved_max > 0.0, "la fixation seule laisse une NPP positive");
    assert!(
        starved_max < 0.5 * fed_max,
        "sans minéralisation ni stock, le plafond ne tient que sur la fixation : {starved_max} vs {fed_max}"
    );
}

#[test]
fn identical_runs_are_bitwise_identical() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut first = earth_like(&field);
    let mut second = earth_like(&field);
    for _ in 0..20 {
        run(&mut first, &field, 1.0).expect("tick");
        run(&mut second, &field, 1.0).expect("tick");
    }
    for (a, b) in first.carbon.stocks.iter().zip(&second.carbon.stocks) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    for (a, b) in first.water.stocks.iter().zip(&second.water.stocks) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

#[test]
fn mixed_layer_follows_revelle_scaling_at_equilibrium() {
    let field = Field::synthetic(5, 0.0, 0.0);
    let config = MatterConfig {
        co2_ppm: 1_000.0,
        volcanic_factor: 1.0,
    };
    let mut state = init(&config, &field.grid, &field.inputs()).expect("init");
    for _ in 0..40 {
        run(&mut state, &field, 1.0).expect("tick");
    }
    let lay = state.carbon.layout;
    let ocean = (0..field.is_ocean.len()).filter(|&i| field.is_ocean[i]);
    let mixed: f64 = ocean.clone().map(|i| state.carbon.get(lay.at(carbon::MIX, i))).sum();
    let coef_sum: f64 = ocean
        .map(|i| carbon::mixed_layer_coef(field.areas_m2[i], f64::from(field.temperature_c[i])))
        .sum();
    let ratio = mixed / (coef_sum * state.co2_ppm().powf(1.0 / super::params::REVELLE));
    assert!((ratio - 1.0).abs() < 1e-2, "rapport au régime de Revelle : {ratio}");
}

#[test]
#[ignore]
fn earth_like_state_stays_within_reference_bands_for_10_kyr() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    let mut worst_drift: f64 = 0.0;
    for _ in 0..1_000 {
        let report = run(&mut state, &field, 10.0).expect("tick");
        worst_drift = worst_drift.max(report.drift.max());
    }
    let c = state.carbon.layout;
    let atm = state.carbon.get(c.glob(carbon::ATM));
    let veg: f64 = field
        .land_cells()
        .iter()
        .map(|&i| state.carbon.get(c.at(carbon::VEG, i)))
        .sum();
    let soil: f64 = field
        .land_cells()
        .iter()
        .map(|&i| state.carbon.get(c.at(carbon::FAST, i)) + state.carbon.get(c.at(carbon::SLOW, i)))
        .sum();
    eprintln!("après 10 kyr : atm {atm:.0} GtC, végétation {veg:.0}, sols {soil:.0}, dérive max {worst_drift:e}");
    // Fenêtres REGLES § T.2 (nutriments) : atmosphère 600-900 GtC, végétation 450-650, sols 1500-2400. Tolérance
    // de ±30 % sur la fenêtre. Valeurs observées au dernier passage (release, L5, 1 000 ticks de 10 ans) :
    // atmosphère ≈ 1 063 GtC (au-dessus de la fenêtre, +18 %), végétation ≈ 536 (dans la fenêtre), sols ≈ 1 419
    // (sous la fenêtre, −5 %). Écart documenté, pas forcé. L'atmosphère n'est pas encore stabilisée à 10 kyr
    // (tendance encore décroissante en fin de run) : à vérifier sur un horizon plus long.
    let within = |value: f64, lo: f64, hi: f64| (0.7 * lo..=1.3 * hi).contains(&value);
    assert!(within(atm, 600.0, 900.0), "atmosphère hors tolérance : {atm}");
    assert!(within(veg, 450.0, 650.0), "végétation hors tolérance : {veg}");
    assert!(within(soil, 1500.0, 2400.0), "sols hors tolérance : {soil}");
    assert!(worst_drift <= DRIFT_TOLERANCE);
}

#[test]
#[ignore]
fn doubled_outgassing_raises_co2_then_settles_without_runaway() {
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    let inputs = field.inputs();
    let start = state.co2_ppm();
    let forcing = Forcing::compute(&state, &inputs);
    let burial = BURIAL_FRACTION * forcing.npp_global_gtc();
    let co2_equilibrium = |volcanic: f64| {
        let net = volcanic - (1.0 - ORGANIC_RECYCLE) * burial;
        PPM_REF * (net / (SILICATE_REF_GTC_YR * forcing.weathering)).powf(1.0 / WEATHER_CO2_EXP)
    };
    let target = co2_equilibrium(2.0 * VOLCANIC_GTC_YR);
    state.volcanic_factor = 2.0;
    let mut peak = start;
    for _ in 0..600 {
        run(&mut state, &field, 1_000.0).expect("tick géologique");
        peak = peak.max(state.co2_ppm());
    }
    let settled = state.co2_ppm();
    eprintln!("CO2 : départ {start:.0} ppm, pic {peak:.0}, final {settled:.0}, cible {target:.0}");
    assert!(peak > start * 1.05, "le dégazage doublé doit faire monter le CO2");
    assert!(
        (settled - target).abs() / target < 0.15,
        "retour à l'équilibre attendu : {settled} vs {target}"
    );
}

/// Cinq ticks annuels sous un pool rayon de `threads` threads : empreinte bit à bit de tous les stocks.
fn stock_bits_with_threads(threads: usize) -> Vec<u64> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("pool rayon");
    let field = Field::synthetic(5, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    pool.install(|| {
        for _ in 0..5 {
            run(&mut state, &field, 1.0).expect("tick");
        }
    });
    state
        .elements()
        .iter()
        .flat_map(|e| e.stocks.iter().map(|v| v.to_bits()))
        .collect()
}

#[test]
fn results_are_bit_identical_for_one_and_four_threads() {
    assert_eq!(stock_bits_with_threads(1), stock_bits_with_threads(4));
}

/// Mesure d'un tick annuel au niveau `level` ; le seuil ne s'applique qu'en release.
fn bench_annual_tick(level: u8, limit_ms: u128) {
    let field = Field::synthetic(level, NPP_LAND_GC_M2_YR, NPP_OCEAN_GC_M2_YR);
    let mut state = earth_like(&field);
    run(&mut state, &field, 1.0).expect("échauffement");
    let start = Instant::now();
    run(&mut state, &field, 1.0).expect("tick annuel");
    let elapsed = start.elapsed();
    eprintln!("banc matter L{level}, tick annuel : {elapsed:?}");
    if !cfg!(debug_assertions) {
        assert!(
            elapsed.as_millis() < limit_ms,
            "tick annuel L{level} trop lent : {elapsed:?}"
        );
    }
}

#[test]
#[ignore]
fn bench_annual_tick_l6_under_100_ms() {
    // Cible finale 40 ms une fois la biologie parallélisée par blocs (TODO.md) ; seuil de non-régression d'ici là.
    bench_annual_tick(6, 100);
}

#[test]
#[ignore]
fn bench_annual_tick_l7_under_350_ms() {
    bench_annual_tick(7, 350);
}
