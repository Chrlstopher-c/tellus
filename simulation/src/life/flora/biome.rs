//! Biome de Whittaker calculé depuis l'état (T, P, couverture) ; albédo de surface.
//! Ordre des classes = mockup/shaders/palettes.js : 0 Inlandsis … 9 Forêt tropicale humide.

/// Valeur de `biome` pour une cellule océanique (aucune classe terrestre).
pub const BIOME_NONE: u8 = 255;
pub const INLANDSIS: u8 = 0;
pub const TOUNDRA: u8 = 1;
pub const TAIGA: u8 = 2;
pub const STEPPE_FROIDE: u8 = 3;
pub const PRAIRIE: u8 = 4;
pub const FORET_TEMPEREE: u8 = 5;
pub const DESERT_CHAUD: u8 = 6;
pub const SAVANE: u8 = 7;
pub const FORET_SAISONNIERE: u8 = 8;
pub const FORET_TROPICALE: u8 = 9;

/// Couverture sous laquelle le sol est nu (classe désertique ou steppe).
const BARE_COVER: f64 = 0.02;
const ALBEDO_OCEAN: f64 = 0.06;
const ALBEDO_BARE: f64 = 0.25;
const ALBEDO_VEGETATION: f64 = 0.15;
const ALBEDO_ICE: f64 = 0.62;

/// Classe de Whittaker : température et précipitations, avec végétation nue réduite au désert ou à la steppe.
pub fn whittaker_class(temp_c: f64, precip_mm: f64, cover: f64) -> u8 {
    if temp_c < -9.0 {
        return INLANDSIS;
    }
    if temp_c < -1.0 {
        return TOUNDRA;
    }
    if cover < BARE_COVER {
        return if temp_c < 6.0 {
            STEPPE_FROIDE
        } else if precip_mm < 500.0 {
            DESERT_CHAUD
        } else {
            SAVANE
        };
    }
    let index = precip_mm / (temp_c + 10.0).max(1.0);
    if temp_c < 6.0 {
        return if precip_mm > 300.0 { TAIGA } else { STEPPE_FROIDE };
    }
    if temp_c < 19.0 {
        return match index {
            i if i < 12.0 => STEPPE_FROIDE,
            i if i < 30.0 => PRAIRIE,
            _ => FORET_TEMPEREE,
        };
    }
    match index {
        i if i < 10.0 => DESERT_CHAUD,
        i if i < 35.0 => SAVANE,
        i if i < 65.0 => FORET_SAISONNIERE,
        _ => FORET_TROPICALE,
    }
}

/// Albédo de surface : mélange couvert/nu, océan, puis glace (0,62).
pub fn surface_albedo(cover: f64, ice_fraction: f64, is_ocean: bool) -> f64 {
    let base = if is_ocean {
        ALBEDO_OCEAN
    } else {
        let c = cover.clamp(0.0, 1.0);
        c * ALBEDO_VEGETATION + (1.0 - c) * ALBEDO_BARE
    };
    let ice = ice_fraction.clamp(0.0, 1.0);
    ice * ALBEDO_ICE + (1.0 - ice) * base
}
