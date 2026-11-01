//! Types fonctionnels de plantes (5 terrestres ordonnés par compétitivité, plus le plancton marin).

/// Types terrestres, ordonnés du plus compétitif au moins compétitif (Tilman).
pub const LAND_PFT: usize = 5;
/// Plancton : seul type des cellules océaniques.
pub const PLANKTON: usize = 5;
pub const PFT_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pft {
    pub name: &'static str,
    pub t_opt_c: f64,
    pub t_width_c: f64,
    pub p_min_mm: f64,
    /// Colonisation maximale, 1/an.
    pub colonisation: f64,
    /// Mortalité de fond, 1/an.
    pub mortality: f64,
    /// Tolérance au feu, 0 à 1.
    pub fire_tolerance: f64,
    /// Temps de séjour de la biomasse, années.
    pub residence_yr: f64,
    /// NPP maximale à pleine adéquation climatique, gC m⁻² an⁻¹.
    pub npp_max_gc_m2_yr: f64,
}

pub const PFTS: [Pft; PFT_COUNT] = [
    Pft {
        name: "Arbre tropical",
        t_opt_c: 26.0,
        t_width_c: 6.0,
        p_min_mm: 1200.0,
        colonisation: 0.08,
        mortality: 0.01,
        fire_tolerance: 0.1,
        residence_yr: 50.0,
        npp_max_gc_m2_yr: 1200.0,
    },
    Pft {
        name: "Arbre tempéré feuillu",
        t_opt_c: 14.0,
        t_width_c: 8.0,
        p_min_mm: 600.0,
        colonisation: 0.15,
        mortality: 0.02,
        fire_tolerance: 0.4,
        residence_yr: 40.0,
        npp_max_gc_m2_yr: 900.0,
    },
    Pft {
        name: "Conifère",
        t_opt_c: 5.0,
        t_width_c: 9.0,
        p_min_mm: 300.0,
        colonisation: 0.15,
        mortality: 0.02,
        fire_tolerance: 0.6,
        residence_yr: 60.0,
        npp_max_gc_m2_yr: 600.0,
    },
    Pft {
        name: "Arbuste",
        t_opt_c: 15.0,
        t_width_c: 12.0,
        p_min_mm: 250.0,
        colonisation: 0.4,
        mortality: 0.08,
        fire_tolerance: 0.5,
        residence_yr: 5.0,
        npp_max_gc_m2_yr: 500.0,
    },
    Pft {
        name: "Herbe",
        t_opt_c: 12.0,
        t_width_c: 15.0,
        p_min_mm: 150.0,
        colonisation: 0.8,
        mortality: 0.25,
        fire_tolerance: 0.9,
        residence_yr: 0.8,
        npp_max_gc_m2_yr: 600.0,
    },
    Pft {
        name: "Plancton",
        t_opt_c: 15.0,
        t_width_c: 20.0,
        p_min_mm: 0.0,
        colonisation: 0.0,
        mortality: 0.0,
        fire_tolerance: 0.0,
        residence_yr: 0.1,
        npp_max_gc_m2_yr: 300.0,
    },
];

/// Adéquation climatique dans [0, 1] : gaussienne en température, seuil sigmoïde en précipitations.
pub fn suitability(pft: &Pft, temp_c: f64, precip_mm: f64) -> f64 {
    let z = (temp_c - pft.t_opt_c) / pft.t_width_c;
    let thermal = (-0.5 * z * z).exp();
    if pft.p_min_mm <= 0.0 {
        return thermal;
    }
    let steepness = 0.1 * pft.p_min_mm + 25.0;
    let water = 1.0 / (1.0 + (-(precip_mm - pft.p_min_mm) / steepness).exp());
    thermal * water
}
