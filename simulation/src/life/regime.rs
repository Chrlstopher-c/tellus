//! Régimes du vivant selon Δt (docs/REGLES.md § 1) : explicite jusqu'à 1 an, relaxé au-delà ; évolution p = 1 − e^(−λΔt).

use crate::clock::Regime;

/// Taux d'événements évolutifs (contrôle de spéciation en régime relaxé), par lignée et par an.
pub const EVOLUTION_RATE_PER_YR: f64 = 1e-3;

/// Vrai pour un régime d'écologie explicite (Δt ≤ 1 an) ; faux pour l'écologie relaxée.
pub fn is_explicit(regime: Regime) -> bool {
    matches!(regime, Regime::Fine | Regime::Annual)
}

/// Probabilité d'un événement évolutif sur le pas : p = 1 − e^(−λΔt).
pub fn evolution_probability(dt_years: f64) -> f64 {
    1.0 - (-EVOLUTION_RATE_PER_YR * dt_years.max(0.0)).exp()
}
