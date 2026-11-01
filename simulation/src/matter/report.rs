//! Rapport d'un tick de matière : scalaires pour les bilans et la chronique (REGLES § 12).

use super::forcing::Forcing;
use super::oxygen;
use super::state::MatterState;
use crate::clock::Regime;

/// Dérive relative de la somme de chaque élément sur le tick (doit rester ≤ 1e-9).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementDrift {
    pub carbon: f64,
    pub nitrogen: f64,
    pub phosphorus: f64,
    pub oxygen: f64,
    pub water: f64,
}

impl ElementDrift {
    /// Pire dérive parmi les éléments.
    pub fn max(&self) -> f64 {
        [self.carbon, self.nitrogen, self.phosphorus, self.oxygen, self.water]
            .into_iter()
            .fold(0.0, f64::max)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatterReport {
    pub tick: u64,
    pub regime: Regime,
    pub co2_ppm: f64,
    pub o2_tmol: f64,
    pub o2_fraction: f64,
    pub npp_land_gtc_yr: f64,
    pub npp_ocean_gtc_yr: f64,
    pub drift: ElementDrift,
    /// Transferts demandés au-delà du stock source (comptés, pas silencieux).
    pub shortfalls: u64,
    /// Valeurs non finies ou négatives refusées dans le registre.
    pub anomalies: u64,
}

/// Instantané des compteurs et des totaux en début de tick, pour calculer les écarts.
pub(crate) struct TickStart {
    pub(crate) totals: [f64; 5],
    pub(crate) shortfalls: u64,
    pub(crate) anomalies: u64,
}

impl TickStart {
    pub(crate) fn capture(state: &MatterState) -> Self {
        Self {
            totals: state.totals(),
            shortfalls: state.elements().iter().map(|e| e.shortfalls).sum(),
            anomalies: state.elements().iter().map(|e| e.anomalies).sum(),
        }
    }
}

fn relative_drift(before: f64, after: f64) -> f64 {
    (after - before).abs() / before.abs().max(f64::MIN_POSITIVE)
}

impl MatterReport {
    pub(crate) fn new(state: &MatterState, start: &TickStart, forcing: &Forcing, tick: u64, regime: Regime) -> Self {
        let after = state.totals();
        let [carbon, nitrogen, phosphorus, o2, water] =
            std::array::from_fn(|k| relative_drift(start.totals[k], after[k]));
        let shortfalls = state.elements().iter().map(|e| e.shortfalls).sum::<u64>() - start.shortfalls;
        let anomalies = state.elements().iter().map(|e| e.anomalies).sum::<u64>() - start.anomalies;
        let o2_tmol = state.o2_tmol();
        Self {
            tick,
            regime,
            co2_ppm: state.co2_ppm(),
            o2_tmol,
            o2_fraction: oxygen::fraction(o2_tmol),
            npp_land_gtc_yr: forcing.npp_land_gtc_yr,
            npp_ocean_gtc_yr: forcing.npp_ocean_gtc_yr,
            drift: ElementDrift {
                carbon,
                nitrogen,
                phosphorus,
                oxygen: o2,
                water,
            },
            shortfalls,
            anomalies,
        }
    }
}
