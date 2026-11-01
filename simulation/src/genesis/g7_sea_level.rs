//! G7 : niveau de la mer par quantile pondéré par l'aire (fraction d'eau exacte), et facteur d'amplitude
//! continental α (relief des continents, autour du niveau zéro de l'isostasie), résolu par dichotomie pour que
//! l'altitude moyenne des terres vaille la cible.

use super::GenesisError;

pub(super) const LAND_MEAN_TARGET_M: f64 = 840.0;
const ALPHA_LO: f64 = 0.05;
const ALPHA_HI: f64 = 20.0;
const BISECTIONS: u32 = 40;

/// Altitude (m) telle que la fraction `water_fraction` de l'aire lui soit inférieure.
pub(super) fn sea_level_m(heights: &[f64], areas: &[f64], water_fraction: f64) -> f64 {
    let mut order: Vec<usize> = (0..heights.len()).collect();
    order.sort_unstable_by(|&a, &b| heights[a].total_cmp(&heights[b]).then(a.cmp(&b)));
    let target = water_fraction * areas.iter().sum::<f64>();
    let mut cumulated = 0.0;
    for &cell in &order {
        cumulated += areas[cell];
        if cumulated >= target {
            return heights[cell];
        }
    }
    heights[order[order.len() - 1]]
}

/// Hauteurs après mise à l'échelle des continents par α (les océans ne bougent pas).
pub(super) fn scaled_continents(heights: &[f64], land: &[bool], alpha: f64) -> Vec<f64> {
    heights
        .iter()
        .zip(land)
        .map(|(&h, &is_land)| if is_land { alpha * h } else { h })
        .collect()
}

/// Hauteurs ramenées au niveau de la mer (zéro).
pub(super) fn shifted_to_sea(heights: &[f64], areas: &[f64], water_fraction: f64) -> Vec<f64> {
    let sea = sea_level_m(heights, areas, water_fraction);
    heights.iter().map(|h| h - sea).collect()
}

/// Altitude moyenne (m, pondérée par l'aire) des cellules émergées.
fn land_mean_m(shifted: &[f64], areas: &[f64]) -> f64 {
    let (mut weighted, mut total) = (0.0, 0.0);
    for (&h, &area) in shifted.iter().zip(areas) {
        if h >= 0.0 {
            weighted += area * h;
            total += area;
        }
    }
    weighted / total
}

/// Facteur α des continents tel que l'altitude moyenne des terres vaille `LAND_MEAN_TARGET_M`.
/// Lève `Calibration` si la cible n'est pas encadrée par `[ALPHA_LO, ALPHA_HI]`.
pub(super) fn alpha_for_land_mean(
    heights: &[f64],
    land: &[bool],
    areas: &[f64],
    water_fraction: f64,
) -> Result<f64, GenesisError> {
    let excess = |alpha: f64| {
        let shifted = shifted_to_sea(&scaled_continents(heights, land, alpha), areas, water_fraction);
        land_mean_m(&shifted, areas) - LAND_MEAN_TARGET_M
    };
    if excess(ALPHA_LO) > 0.0 || excess(ALPHA_HI) < 0.0 {
        return Err(GenesisError::Calibration {
            target_m: LAND_MEAN_TARGET_M,
        });
    }
    let (mut lo, mut hi) = (ALPHA_LO, ALPHA_HI);
    for _ in 0..BISECTIONS {
        let mid = 0.5 * (lo + hi);
        if excess(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}
