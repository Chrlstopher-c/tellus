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
