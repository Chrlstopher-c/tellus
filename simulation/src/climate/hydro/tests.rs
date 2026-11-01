use super::*;
use crate::climate::body::body;
use crate::climate::state::{init, step, ClimateConfig, ClimateInputs, ClimateState, Surface};
use crate::clock::{regime, Regime, System};
use crate::shared::grid::Grid;
use crate::shared::units::{LATENT_HEAT_VAPORIZATION_J_KG, R_EARTH_M, SECONDS_PER_YEAR};
use std::time::Instant;

/// Terre synthétique : deux continents (lon, lat en degrés), crête méridienne de 2 000 m optionnelle sur le premier.
struct Terra {
    grid: Grid,
    elevation: Vec<f64>,
    ocean: Vec<bool>,
}

const RIDGE_LON_DEG: f64 = -72.0;

fn lon_lat_deg(c: [f64; 3]) -> (f64, f64) {
    (c[2].atan2(c[0]).to_degrees(), c[1].asin().to_degrees())
}

fn is_land(lon: f64, lat: f64) -> bool {
    let first = (-110.0..-35.0).contains(&lon) && lat.abs() <= 65.0;
    let second = (10.0..60.0).contains(&lon) && (-40.0..70.0).contains(&lat);
    first || second
}

fn terra(level: u8, ridge: bool) -> Terra {
    let grid = Grid::new(level, R_EARTH_M);
    let mut elevation = vec![0.0; grid.cell_count()];
    let mut ocean = vec![true; grid.cell_count()];
    for (i, c) in grid.centers().iter().enumerate() {
        let (lon, lat) = lon_lat_deg(*c);
        ocean[i] = !is_land(lon, lat);
        if ridge && !ocean[i] && lat.abs() <= 30.0 && (-110.0..-35.0).contains(&lon) {
            elevation[i] = 2_000.0 * (-((lon - RIDGE_LON_DEG) / 5.0).powi(2)).exp();
        }
    }
    Terra { grid, elevation, ocean }
}

fn earth_config() -> ClimateConfig {
    ClimateConfig::earth()
}

/// Atmosphère à l'équilibre thermique (10 000 ans), puis quelques pas de 11 ans pour que le seau et le transport
/// suivent. Chaque pas de 11 ans déclenche un recalcul hydrologique.
fn spun_up(t: &Terra, config: &ClimateConfig) -> (ClimateState, Option<HydroReport>) {
    let surface = Surface {
        elevation_m: &t.elevation,
        is_ocean: &t.ocean,
    };
    let mut state = init(&t.grid, &body("earth").expect("terre"), &surface, config).expect("init");
    let inputs = ClimateInputs {
        co2_ppm: 280.0,
        surface_albedo: None,
    };
    step(
        &mut state,
        &t.grid,
        &inputs,
        10_000.0,
        regime(System::Climate, 10_000.0),
        0,
        0,
    )
    .expect("équilibre");
    let mut last = None;
    for _ in 0..3 {
        last = step(&mut state, &t.grid, &inputs, 11.0, Regime::Secular, 0, 0)
            .expect("pas")
            .hydro;
    }
    (state, last)
}

fn area_mean_of(t: &Terra, values: &[f64], keep: impl Fn(f64, f64) -> bool) -> f64 {
    let (sum, weight) =
        t.grid
            .centers()
            .iter()
            .zip(values)
            .zip(t.grid.areas_m2())
            .fold((0.0, 0.0), |(s, w), ((c, v), a)| {
                let (lon, lat) = lon_lat_deg(*c);
                if keep(lon, lat) {
                    (s + a * v, w + a)
                } else {
                    (s, w)
                }
            });
    sum / weight
}

fn lat_band_mean(t: &Terra, values: &[f64], lo: f64, hi: f64) -> f64 {
    area_mean_of(t, values, |_, lat| (lo..hi).contains(&lat.abs()))
}

/// V-18 : e_s = 6,11 hPa (0 °C), 17,04 (15 °C), 42,4 (30 °C).
#[test]
fn v18_saturation_vapour_pressure_matches_bolton_points() {
    assert!((saturation_vapour_hpa(0.0) - 6.11).abs() <= 0.01);
    assert!((saturation_vapour_hpa(15.0) - 17.04).abs() <= 0.02);
    assert!((saturation_vapour_hpa(30.0) - 42.4).abs() <= 0.1);
}

/// V-19 : d ln e_s / dT à 15 °C = 6,4 %/K.
#[test]
fn v19_clausius_clapeyron_slope_at_15_degrees_is_6_4_percent_per_kelvin() {
    let slope = clausius_clapeyron_slope(15.0);
    assert!((slope - 0.064).abs() <= 0.002, "pente {slope} K⁻¹");
    let numeric = (saturation_vapour_hpa(15.001).ln() - saturation_vapour_hpa(14.999).ln()) / 0.002;
    assert!((numeric - slope).abs() < 1e-6, "dérivée numérique {numeric}");
}

/// V-20 : 1 000 mm an⁻¹ d'évaporation = 79 ± 2 W m⁻² de flux latent.
#[test]
fn v20_one_metre_per_year_is_79_w_per_square_metre() {
    let kg_m2_s = 1000.0 / SECONDS_PER_YEAR;
    let flux = kg_m2_s * LATENT_HEAT_VAPORIZATION_J_KG;
    assert!((flux - 79.0).abs() <= 2.0, "flux latent {flux} W/m²");
}

/// Conservation exacte : le flux de l'arête (i→j) est l'opposé de celui de (j→i).
#[test]
fn directed_flux_is_exactly_antisymmetric() {
    let t = terra(4, false);
    let mesh = Mesh::new(&t.grid).expect("maillage");
    let winds: Vec<[f64; 3]> = t.grid.centers().iter().map(|c| [c[2], -c[0], 0.3 * c[1]]).collect();
    let flux = transport::directed_flux(&mesh, &winds);
    let offsets = mesh.edge_offsets();
    let neighbours = mesh.neighbor_indices();
    for i in 0..mesh.cell_count() {
        for k in offsets[i] as usize..offsets[i + 1] as usize {
            let j = neighbours[k] as usize;
            let back = (offsets[j] as usize..offsets[j + 1] as usize)
                .find(|&m| neighbours[m] as usize == i)
                .expect("arête réciproque");
            let sum = flux[k] + flux[back];
            assert!(
                sum.abs() <= 1e-12 * flux[k].abs().max(1.0),
                "({i},{j}) : {} + {}",
                flux[k],
                flux[back]
            );
        }
    }
}

/// V-21 : |P̄ − Ē| / P̄ < 1 % à l'équilibre hydrologique.
#[test]
fn v21_water_balance_closes_within_one_percent() {
    let t = terra(4, false);
    let (_, report) = spun_up(&t, &earth_config());
    let report = report.expect("recalcul hydrologique");
    let gap = (report.mean_precip_mm_yr - report.mean_evap_mm_yr).abs() / report.mean_precip_mm_yr;
    assert!(
        gap < 0.01,
        "P̄ {} vs Ē {} mm/an",
        report.mean_precip_mm_yr,
        report.mean_evap_mm_yr
    );
    assert_eq!(report.unconverged_seasons, 0, "{report:?}");
}

/// V-22 : P̄ ≈ 1 000 mm an⁻¹ (± 100), maximum près de l'équateur, P(30°) < 0,5·P(0°).
#[test]
fn v22_global_precipitation_is_about_one_metre_with_itcz_and_subtropical_dry_belt() {
    let t = terra(5, false);
    let (state, report) = spun_up(&t, &earth_config());
    let mean = report.expect("recalcul").mean_precip_mm_yr;
    assert!((mean - 1000.0).abs() <= 100.0, "P̄ = {mean} mm/an");
    let equator = lat_band_mean(&t, &state.precip_mm_yr, 0.0, 5.0);
    let subtropics = lat_band_mean(&t, &state.precip_mm_yr, 27.0, 33.0);
    assert!(subtropics < 0.5 * equator, "P(30°) {subtropics} vs P(0°) {equator}");
    let zonal_peak = lat_band_mean(&t, &state.precip_mm_yr, 0.0, 10.0);
    let outer = lat_band_mean(&t, &state.precip_mm_yr, 10.0, 90.0);
    assert!(zonal_peak > outer, "maximum hors de ±10° : {zonal_peak} vs {outer}");
}

/// V-23 : crête de 2 000 m, pluie au vent et ombre sous le vent (côté ouest, en alizés d'est).
#[test]
fn v23_two_kilometre_ridge_casts_a_rain_shadow_to_the_lee() {
    let ridge = terra(5, true);
    let (state, _) = spun_up(&ridge, &earth_config());
    let windward = area_mean_of(&ridge, &state.precip_mm_yr, |lon, lat| {
        (-68.0..-45.0).contains(&lon) && (8.0..25.0).contains(&lat.abs())
    });
    let lee = area_mean_of(&ridge, &state.precip_mm_yr, |lon, lat| {
        (-100.0..-76.0).contains(&lon) && (8.0..25.0).contains(&lat.abs())
    });
    assert!(lee < 0.5 * windward, "sous le vent {lee} vs au vent {windward} mm/an");
}

/// Sans océan, pas d'évaporation ni de précipitation ; sol et glace de mer restent cohérents.
#[test]
fn dry_world_without_ocean_has_no_precipitation_and_no_sea_ice() {
    let mut t = terra(3, false);
    t.ocean = vec![false; t.grid.cell_count()];
    let (state, report) = spun_up(&t, &earth_config());
    let report = report.expect("recalcul");
    assert!(report.mean_precip_mm_yr.abs() < 1e-9, "P̄ {}", report.mean_precip_mm_yr);
    assert!(state.sea_ice_fraction.iter().all(|&f| f == 0.0));
    assert!(state.soil_moisture.iter().all(|&s| (0.0..=1.0).contains(&s)));
}

/// Les diagnostics ne se recalculent pas à chaque pas : un pas de 1 mois sans dérive thermique n'en déclenche pas.
#[test]
fn diagnostics_are_not_recomputed_on_every_tick() {
    let t = terra(3, false);
    let surface = Surface {
        elevation_m: &t.elevation,
        is_ocean: &t.ocean,
    };
    let mut state = init(&t.grid, &body("earth").expect("terre"), &surface, &earth_config()).expect("init");
    let inputs = ClimateInputs {
        co2_ppm: 280.0,
        surface_albedo: None,
    };
    let month = 1.0 / 12.0;
    let first = step(&mut state, &t.grid, &inputs, month, Regime::Fine, 0, 0).expect("pas");
    assert!(first.hydro.is_none(), "un mois ne doit pas recalculer");
}

/// Déterminisme : même état après un recalcul, à 1 et à 6 threads, bit à bit.
#[test]
fn hydrology_is_bitwise_identical_for_one_and_six_threads() {
    let run = |threads: usize| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("pool")
            .install(|| {
                let t = terra(4, true);
                let (state, _) = spun_up(&t, &earth_config());
                state.precip_mm_yr
            })
    };
    let (single, parallel) = (run(1), run(6));
    assert!(single.iter().zip(&parallel).all(|(a, b)| a.to_bits() == b.to_bits()));
}

/// Banc : coût du recalcul hydrologique à L6 = (pas avec cycle de l'eau) − (même pas sans), 6 cœurs. Cible < 150 ms.
#[test]
#[ignore = "banc release : recalcul diagnostic L6 < 150 ms"]
fn bench_hydro_diagnostic_l6_under_150_ms() {
    let t = terra(6, true);
    let inputs = ClimateInputs {
        co2_ppm: 280.0,
        surface_albedo: None,
    };
    let timed = |config: &ClimateConfig| {
        let surface = Surface {
            elevation_m: &t.elevation,
            is_ocean: &t.ocean,
        };
        let mut state = init(&t.grid, &body("earth").expect("terre"), &surface, config).expect("init");
        let start = Instant::now();
        let report = step(&mut state, &t.grid, &inputs, 11.0, Regime::Secular, 0, 0).expect("pas");
        (start.elapsed().as_secs_f64() * 1e3, report.hydro)
    };
    let with_water = rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build()
        .expect("pool")
        .install(|| timed(&earth_config()));
    let without = rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build()
        .expect("pool")
        .install(|| {
            timed(&ClimateConfig {
                hydrology: false,
                ..earth_config()
            })
        });
    let cost = with_water.0 - without.0;
    println!("BANC hydro L6 : {cost:.1} ms, {:?}", with_water.1);
    assert!(cost < 150.0, "recalcul hydrologique L6 : {cost} ms");
}

#[test]
#[ignore = "sonde temporaire"]
fn probe_zonal_profile() {
    let t = terra(4, false);
    let (state, report) = spun_up(&t, &earth_config());
    println!("REPORT {report:?}");
    for band in 0..9 {
        let (lo, hi) = (f64::from(band) * 10.0, f64::from(band) * 10.0 + 10.0);
        let land = area_mean_of(&t, &state.precip_mm_yr, |lon, lat| {
            (lo..hi).contains(&lat.abs()) && is_land(lon, lat)
        });
        let sea = area_mean_of(&t, &state.precip_mm_yr, |lon, lat| {
            (lo..hi).contains(&lat.abs()) && !is_land(lon, lat)
        });
        let temp = area_mean_of(&t, &state.temperature_c, |_, lat| (lo..hi).contains(&lat.abs()));
        let wu = area_mean_of(&t, &state.wind_u, |_, lat| (lo..hi).contains(&lat.abs()));
        let wv = area_mean_of(&t, &state.wind_v, |_, lat| (lo..hi).contains(&lat.abs()));
        println!("BAND {lo:>3}-{hi:<3} P_land {land:8.1} P_sea {sea:8.1} T {temp:6.2} u {wu:6.2} v {wv:6.2}");
    }
}
