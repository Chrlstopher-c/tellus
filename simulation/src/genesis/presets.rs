//! Corps réels jouables (Terre, Mars, Vénus, Lune, Mercure) : relief issu de MNT publics réduits à la grille par
//! `scripts/dem/build.sh`. Un fichier par corps et niveau : f32 little-endian, une altitude (m) par cellule.
//! Le niveau 6 est embarqué dans le binaire ; le niveau 7 est lu sur disque.

use std::path::PathBuf;

use super::{GenesisError, Planet, Plate};
use crate::shared::grid::Grid;

const LEVEL_EMBEDDED: u8 = 6;
const LEVEL_FILE: u8 = 7;
const FROZEN_PLATE: Plate = Plate {
    pole: [0.0, 0.0, 1.0],
    omega_rad_s: 0.0,
    continental: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Body {
    Mercury,
    Venus,
    Earth,
    Mars,
    Moon,
}

impl Body {
    fn from_name(name: &str) -> Result<Self, GenesisError> {
        match name {
            "mercury" => Ok(Self::Mercury),
            "venus" => Ok(Self::Venus),
            "earth" => Ok(Self::Earth),
            "mars" => Ok(Self::Mars),
            "moon" => Ok(Self::Moon),
            _ => Err(GenesisError::InvalidParam("corps réel inconnu")),
        }
    }

    fn stem(self) -> &'static str {
        match self {
            Self::Mercury => "mercury",
            Self::Venus => "venus",
            Self::Earth => "earth",
            Self::Mars => "mars",
            Self::Moon => "moon",
        }
    }

    fn has_ocean(self) -> bool {
        matches!(self, Self::Earth)
    }

    fn embedded_level_6(self) -> &'static [u8] {
        match self {
            Self::Mercury => include_bytes!("../../data/dem/mercury_L6.f32"),
            Self::Venus => include_bytes!("../../data/dem/venus_L6.f32"),
            Self::Earth => include_bytes!("../../data/dem/earth_L6.f32"),
            Self::Mars => include_bytes!("../../data/dem/mars_L6.f32"),
            Self::Moon => include_bytes!("../../data/dem/moon_L6.f32"),
        }
    }
}

fn decode(bytes: &[u8], cells: usize) -> Result<Vec<f32>, GenesisError> {
    if bytes.len() != cells * 4 {
        return Err(GenesisError::Inconsistent(
            "taille du MNT différente du nombre de cellules",
        ));
    }
    let (words, _) = bytes.as_chunks::<4>();
    let values: Vec<f32> = words.iter().map(|w| f32::from_le_bytes(*w)).collect();
    match values.iter().position(|v| !v.is_finite()) {
        Some(cell) => Err(GenesisError::NonFinite {
            field: "elevation_m",
            cell,
        }),
        None => Ok(values),
    }
}

fn read_file_level(body: Body) -> Result<Vec<u8>, GenesisError> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/dem")
        .join(format!("{}_L{LEVEL_FILE}.f32", body.stem()));
    std::fs::read(path).map_err(|_| GenesisError::Inconsistent("MNT de niveau 7 absent ou illisible"))
}

fn planet_from_elevation(body: Body, elevation_m: Vec<f32>) -> Planet {
    let cells = elevation_m.len();
    let crust = elevation_m
        .iter()
        .map(|&h| if !body.has_ocean() || h > 0.0 { 1.0 } else { 0.0 })
        .collect();
    Planet {
        elevation_m,
        crust,
        plate: vec![0.0; cells],
        age_ma: vec![0.0; cells],
        plates: vec![FROZEN_PLATE],
    }
}

/// Planète réelle sur la grille donnée : relief du MNT, croûte continentale où l'altitude est > 0 (Terre seulement,
/// les autres corps n'ont pas d'océan), une plaque figée. Erreur si le corps est inconnu ou le niveau sans MNT.
pub fn real_body(grid: &Grid, name: &str) -> Result<Planet, GenesisError> {
    let body = Body::from_name(name)?;
    let cells = grid.cell_count();
    let elevation = match grid.level() {
        LEVEL_EMBEDDED => decode(body.embedded_level_6(), cells)?,
        LEVEL_FILE => decode(&read_file_level(body)?, cells)?,
        _ => return Err(GenesisError::InvalidParam("niveau de grille sans MNT réel")),
    };
    Ok(planet_from_elevation(body, elevation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::units::R_EARTH_M;
    use std::path::Path;

    const NAMES: [&str; 5] = ["mercury", "venus", "earth", "mars", "moon"];
    const EARTH_LAND_TARGET: f64 = 0.29;
    const EARTH_LAND_TOLERANCE: f64 = 0.03;

    fn grid(level: u8) -> Grid {
        Grid::new(level, R_EARTH_M)
    }

    fn area_weighted_mean(grid: &Grid, keep: impl Fn(usize) -> bool, value: impl Fn(usize) -> f64) -> f64 {
        let (mut sum, mut area) = (0.0, 0.0);
        for (cell, &a) in grid.areas_m2().iter().enumerate() {
            if keep(cell) {
                sum += a * value(cell);
                area += a;
            }
        }
        sum / area
    }

    fn land_fraction(grid: &Grid, planet: &Planet) -> f64 {
        area_weighted_mean(grid, |_| true, |c| f64::from(planet.crust[c]))
    }

    #[test]
    fn presets_every_body_loads_at_l6() {
        let grid = grid(6);
        for name in NAMES {
            let planet = real_body(&grid, name).unwrap_or_else(|e| panic!("{name} : {e}"));
            assert_eq!(planet.elevation_m.len(), grid.cell_count(), "{name}");
            assert!(
                planet.elevation_m.iter().all(|h| h.is_finite()),
                "{name} : valeur non finie"
            );
            assert_eq!(planet.plates.len(), 1, "{name} : une seule plaque figée");
        }
    }

    #[test]
    fn presets_earth_land_fraction() {
        let grid = grid(6);
        let planet = real_body(&grid, "earth").expect("Terre");
        let land = land_fraction(&grid, &planet);
        assert!(
            (land - EARTH_LAND_TARGET).abs() <= EARTH_LAND_TOLERANCE,
            "fraction de terres {land:.3} hors de 0,29 ± 0,03"
        );
    }

    #[test]
    fn presets_mars_north_lower_than_south() {
        let grid = grid(6);
        let planet = real_body(&grid, "mars").expect("Mars");
        let elevation = |c: usize| f64::from(planet.elevation_m[c]);
        let north = area_weighted_mean(&grid, |c| grid.centers()[c][2] > 0.0, elevation);
        let south = area_weighted_mean(&grid, |c| grid.centers()[c][2] < 0.0, elevation);
        assert!(north < south, "Mars : nord {north:.0} m, sud {south:.0} m");
    }

    #[test]
    fn presets_are_deterministic() {
        let grid = grid(6);
        for name in NAMES {
            let first = real_body(&grid, name).expect(name);
            let second = real_body(&grid, name).expect(name);
            assert_eq!(first, second, "{name}");
        }
    }

    #[test]
    fn presets_reject_unknown_body_and_level() {
        let grid6 = grid(6);
        assert!(matches!(real_body(&grid6, "titan"), Err(GenesisError::InvalidParam(_))));
        let grid4 = grid(4);
        assert!(matches!(real_body(&grid4, "earth"), Err(GenesisError::InvalidParam(_))));
    }

    #[test]
    fn presets_level_7_loads_from_file() {
        let grid = grid(7);
        let planet = real_body(&grid, "earth").expect("Terre L7");
        assert_eq!(planet.elevation_m.len(), grid.cell_count());
        let land = land_fraction(&grid, &planet);
        assert!(
            (land - EARTH_LAND_TARGET).abs() <= EARTH_LAND_TOLERANCE,
            "L7 : {land:.3}"
        );
    }

    /// Export des centres pour `scripts/dem/` : `TELLUS_CENTERS_DIR=<dir> cargo test export_grid_centers -- --ignored`.
    #[test]
    #[ignore = "export hors ligne, lancé par scripts/dem/build.sh"]
    fn export_grid_centers() {
        let dir = std::env::var("TELLUS_CENTERS_DIR").expect("TELLUS_CENTERS_DIR requis");
        for level in [LEVEL_EMBEDDED, LEVEL_FILE] {
            let grid = grid(level);
            let mut bytes = Vec::with_capacity(grid.cell_count() * 32);
            for (center, area_m2) in grid.centers().iter().zip(grid.areas_m2()) {
                for value in [center[0], center[1], center[2], area_m2 / (R_EARTH_M * R_EARTH_M)] {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
            let target = Path::new(&dir).join(format!("grid_L{level}.f64"));
            std::fs::write(&target, bytes).expect("écriture des centres");
        }
    }
}
