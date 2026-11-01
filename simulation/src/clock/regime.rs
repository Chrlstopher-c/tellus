//! Régime de chaque système selon la durée du tick Δt (REGLES § 1, tableau).

/// Système du monde dont le régime est choisi à chaque tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    Climate,
    Matter,
    Life,
    Geology,
}

/// Régime d'un système pour une durée de tick donnée. Le comportement détaillé (sous-pas, super-tick,
/// équilibre) appartient au domaine consommateur ; ce module ne fixe que la sélection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    /// Δt ≤ 1 mois : pas mensuel explicite (climat, matière) ou annuel découpé (vivant).
    Fine,
    /// 1 mois < Δt ≤ 1 an : 12 sous-pas mensuels (climat) ou annuel exact (matière, vivant).
    Annual,
    /// 1 an < Δt ≤ 1 000 ans : moyenne annuelle (climat), annuel + super-tick géologique (matière),
    /// écologie à l'équilibre (vivant).
    Secular,
    /// Δt > 1 000 ans : relaxation (climat), géologique seul (matière), évolution à sous-pas plafonné (vivant).
    Geological,
    /// Géologie : figée au MVP, quelle que soit la durée.
    Frozen,
}

const MONTH_YEARS: f64 = 1.0 / 12.0;
const YEAR_YEARS: f64 = 1.0;
const MILLENNIUM_YEARS: f64 = 1_000.0;

/// Régime d'un système pour un tick de durée `dt_years` (années simulées, finie et positive).
pub fn regime(system: System, dt_years: f64) -> Regime {
    match system {
        System::Geology => Regime::Frozen,
        System::Climate | System::Matter | System::Life => by_duration(dt_years),
    }
}

fn by_duration(dt_years: f64) -> Regime {
    if dt_years <= MONTH_YEARS {
        Regime::Fine
    } else if dt_years <= YEAR_YEARS {
        Regime::Annual
    } else if dt_years <= MILLENNIUM_YEARS {
        Regime::Secular
    } else {
        Regime::Geological
    }
}
