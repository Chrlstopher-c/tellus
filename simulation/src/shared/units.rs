//! Constantes physiques en SI ; les conversions n'existent qu'à l'affichage.

/// Constante de Stefan-Boltzmann, W m⁻² K⁻⁴.
pub const SIGMA: f64 = 5.670_374_419e-8;
/// Rayon moyen de la Terre, m.
pub const R_EARTH_M: f64 = 6.371e6;
/// Pesanteur standard, m s⁻².
pub const G_STANDARD: f64 = 9.806_65;
/// Constante des gaz parfaits, J mol⁻¹ K⁻¹.
pub const GAS_CONSTANT: f64 = 8.314_462_618;
/// Constante de Boltzmann, eV K⁻¹ (énergies d'activation MTE en eV).
pub const BOLTZMANN_EV: f64 = 8.617_333_262e-5;
/// Secondes dans un jour.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
/// Secondes dans une année julienne.
pub const SECONDS_PER_YEAR: f64 = 365.25 * SECONDS_PER_DAY;
/// Secondes dans un million d'années.
pub const SECONDS_PER_MA: f64 = 1.0e6 * SECONDS_PER_YEAR;
/// Masse volumique du manteau, kg m⁻³.
pub const RHO_MANTLE: f64 = 3_300.0;
/// Masse volumique de la croûte continentale, kg m⁻³.
pub const RHO_CONTINENTAL_CRUST: f64 = 2_700.0;
/// Épaisseur de croûte continentale de référence (niveau zéro de l'isostasie d'Airy), m.
pub const CRUST_REFERENCE_THICKNESS_M: f64 = 35_000.0;
/// GDH1 : profondeur de la dorsale (âge nul), m.
pub const GDH1_RIDGE_DEPTH_M: f64 = 2_600.0;
/// GDH1 : coefficient de la branche en racine carrée de l'âge, m Ma⁻¹ᐟ².
pub const GDH1_SQRT_COEFF_M: f64 = 365.0;
/// GDH1 : âge de raccord entre les deux branches, Ma.
pub const GDH1_TRANSITION_MA: f64 = 20.0;
/// GDH1 : profondeur asymptotique de la branche exponentielle, m.
pub const GDH1_DEEP_DEPTH_M: f64 = 5_651.0;
/// GDH1 : amplitude de la branche exponentielle, m.
pub const GDH1_DEEP_AMP_M: f64 = 2_473.0;
/// GDH1 : taux de la branche exponentielle, Ma⁻¹.
pub const GDH1_DECAY_PER_MA: f64 = 0.0278;
/// Capacité thermique volumique de l'eau, J m⁻³ K⁻¹ (ρ·c, équivalent eau des capacités de surface).
pub const WATER_VOLUMETRIC_HEAT_CAPACITY: f64 = 4.18e6;
/// Grammes par gigatonne (1 Gt = 10¹⁵ g).
pub const GRAMS_PER_GT: f64 = 1.0e15;
/// Lame d'eau sur une surface : 1 mm sur 1 m² = 1e-12 km³.
pub const KM3_PER_MM_M2: f64 = 1.0e-12;
/// ppm de CO₂ atmosphérique par GtC (2,124 GtC/ppm, docs/recherche/nutriments.md § 0).
pub const GTC_PER_PPM: f64 = 2.124;
