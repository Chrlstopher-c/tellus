use super::*;
use crate::shared::units::R_EARTH_M;

const OMEGA_EARTH: f64 = 2.0 * PI / (EARTH_SIDEREAL_DAY_HOURS * 3600.0);

fn earth_circulation(centre_deg: f64) -> Circulation {
    Circulation::new(centre_deg.to_radians(), OMEGA_EARTH, R_EARTH_M, 1.0)
}

/// V-24 (forme seulement) : Held–Hou sans friction à 30° sur Terre, 134 ± 2 m s⁻¹.
#[test]
fn v24_held_hou_upper_wind_at_30_degrees_is_134_ms() {
    let u = held_hou_upper_wind(30_f64.to_radians(), OMEGA_EARTH, R_EARTH_M);
    assert!((u - 134.0).abs() <= 2.0, "u_haut(30°) = {u} m/s");
}

#[test]
fn hadley_width_is_30_degrees_on_earth_and_scales_as_omega_power() {
    assert!((hadley_half_width(1.0).to_degrees() - 30.0).abs() < 1e-9);
    let slow = hadley_half_width(0.5).to_degrees();
    assert!((slow - 30.0 * 2.0_f64.sqrt()).abs() < 1e-9, "Ω/2 : {slow}°");
    let fast = hadley_half_width(2.0).to_degrees();
    assert!((fast - 30.0 / 2.0_f64.sqrt()).abs() < 1e-9, "2Ω : {fast}°");
}

#[test]
fn hadley_width_is_bounded_for_extreme_rotation() {
    assert!((hadley_half_width(1e-6).to_degrees() - HADLEY_HALF_WIDTH_MAX_DEG).abs() < 1e-9);
    assert!((hadley_half_width(1e6).to_degrees() - HADLEY_HALF_WIDTH_MIN_DEG).abs() < 1e-9);
}

#[test]
fn earth_trade_winds_are_about_six_metres_per_second() {
    let circ = earth_circulation(0.0);
    let (u, _) = surface_wind(&circ, 15_f64.to_radians());
    assert!((-6.5..-5.5).contains(&u), "alizés : u = {u} m/s");
}

#[test]
fn trades_blow_east_to_west_and_westerlies_west_to_east() {
    let circ = earth_circulation(0.0);
    for lat in [-25.0_f64, -15.0, 15.0, 25.0] {
        assert!(surface_wind(&circ, lat.to_radians()).0 < 0.0, "alizés à {lat}°");
    }
    for lat in [-50.0_f64, -45.0, 45.0, 50.0] {
        assert!(surface_wind(&circ, lat.to_radians()).0 > 0.0, "westerlies à {lat}°");
    }
    for lat in [-75.0_f64, 75.0] {
        assert!(surface_wind(&circ, lat.to_radians()).0 < 0.0, "easterlies à {lat}°");
    }
}

#[test]
fn meridional_wind_converges_on_the_intertropical_convergence_zone() {
    let circ = earth_circulation(0.0);
    assert!(
        surface_wind(&circ, (-10_f64).to_radians()).1 > 0.0,
        "sud de la ZCIT : vers le nord"
    );
    assert!(
        surface_wind(&circ, 10_f64.to_radians()).1 < 0.0,
        "nord de la ZCIT : vers le sud"
    );
    assert!(surface_wind(&circ, 0.0).1.abs() < 1e-12, "axe de la ZCIT : v nul");
}

#[test]
fn surface_wind_is_continuous_at_cell_edges() {
    let circ = earth_circulation(7.0);
    let edges = [
        circ.centre_rad - circ.hadley_half_rad,
        circ.centre_rad + circ.hadley_half_rad,
        circ.centre_rad + circ.ferrel_edge_rad,
        circ.centre_rad - circ.ferrel_edge_rad,
    ];
    for edge in edges {
        let below = surface_wind(&circ, edge - 1e-9);
        let above = surface_wind(&circ, edge + 1e-9);
        assert!(
            (below.0 - above.0).abs() < 1e-3 && (below.1 - above.1).abs() < 1e-3,
            "bord {edge}"
        );
    }
}

#[test]
fn intertropical_convergence_zone_follows_its_centre() {
    let shifted = earth_circulation(10.0);
    assert!(surface_wind(&shifted, 10_f64.to_radians()).1.abs() < 1e-12);
    assert!(
        surface_wind(&shifted, 0.0).1 > 0.0,
        "au sud de la ZCIT décalée, le vent va vers le nord"
    );
}
