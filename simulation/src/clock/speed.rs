//! Vitesses de simulation (REGLES § 1) exprimées en années simulées par seconde murale.

use crate::shared::units::{SECONDS_PER_DAY, SECONDS_PER_YEAR};

/// Vitesse demandée par le joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Pause,
    Day,
    Year,
    Century,
    TenKyr,
    Myr,
    TenMyr,
}

impl Speed {
    /// Années simulées par seconde murale.
    pub fn years_per_second(self) -> f64 {
        match self {
            Self::Pause => 0.0,
            Self::Day => SECONDS_PER_DAY / SECONDS_PER_YEAR,
            Self::Year => 1.0,
            Self::Century => 100.0,
            Self::TenKyr => 10_000.0,
            Self::Myr => 1_000_000.0,
            Self::TenMyr => 10_000_000.0,
        }
    }
}
