//! Ordonnancement des ticks : Δt = vitesse / ticks par seconde. Quand le calcul ne suit pas, la vitesse
//! effective baisse ; le temps simulé n'avance jamais par saut.

use super::speed::Speed;

const TARGET_TPS: f64 = 10.0;
const MIN_TPS: f64 = 2.0;
const MAX_TPS: f64 = 20.0;
/// Rattrapage maximal, en ticks : le temps mural au-delà est perdu, pas rejoué en rafale.
const MAX_BACKLOG_TICKS: f64 = 2.0;
/// Poids du dernier coût mesuré dans la moyenne mobile exponentielle.
const COST_SMOOTHING: f64 = 0.25;

/// Tick à exécuter : durée simulée en années.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub index: u64,
    pub dt_years: f64,
}

/// Décide quand et de quelle durée sont les ticks, à partir du temps mural et du coût mesuré.
#[derive(Debug, Clone)]
pub struct Scheduler {
    speed: Speed,
    tps: f64,
    sim_years: f64,
    ticks: u64,
    owed_s: f64,
    now_s: f64,
    busy_until_s: f64,
    cost_s: Option<f64>,
}

impl Scheduler {
    /// Ordonnanceur à la cadence cible de 10 ticks par seconde.
    pub fn new(speed: Speed) -> Self {
        Self::with_rate(speed, TARGET_TPS)
    }

    /// Ordonnanceur à la cadence demandée, ramenée dans [2, 20] ticks par seconde.
    pub fn with_rate(speed: Speed, tps: f64) -> Self {
        Self {
            speed,
            tps: clamp_rate(tps),
            sim_years: 0.0,
            ticks: 0,
            owed_s: 0.0,
            now_s: 0.0,
            busy_until_s: 0.0,
            cost_s: None,
        }
    }

    /// Change la vitesse demandée ; prend effet au prochain tick.
    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
    }

    /// Vitesse demandée.
    pub fn speed(&self) -> Speed {
        self.speed
    }

    /// Temps simulé cumulé, en années.
    pub fn sim_years(&self) -> f64 {
        self.sim_years
    }

    /// Nombre de ticks produits.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Temps mural écoulé depuis l'appel précédent, calcul compris. Ignoré si non fini ou négatif.
    pub fn accrue(&mut self, wall_s: f64) {
        if !is_duration(wall_s) {
            return;
        }
        self.now_s += wall_s;
        if self.speed == Speed::Pause {
            self.owed_s = 0.0;
            return;
        }
        self.owed_s = (self.owed_s + wall_s).min(MAX_BACKLOG_TICKS * self.period_s());
    }

    /// Produit le prochain tick si le temps dû est suffisant et si le calcul précédent est terminé.
    pub fn next_tick(&mut self) -> Option<Tick> {
        let period = self.period_s();
        if self.speed == Speed::Pause || self.owed_s < period || self.now_s < self.busy_until_s {
            return None;
        }
        let dt_years = self.speed.years_per_second() * period;
        let tick = Tick {
            index: self.ticks,
            dt_years,
        };
        self.owed_s -= period;
        self.sim_years += dt_years;
        self.ticks += 1;
        Some(tick)
    }

    /// Durée réelle, en secondes, du tick qui vient d'être exécuté. Ignoré si non fini ou négatif.
    pub fn record_cost(&mut self, cost_s: f64) {
        if !is_duration(cost_s) {
            return;
        }
        self.busy_until_s = self.now_s + cost_s;
        self.cost_s = Some(match self.cost_s {
            None => cost_s,
            Some(previous) => previous + COST_SMOOTHING * (cost_s - previous),
        });
    }

    /// Vitesse réellement tenue, en années par seconde murale : la demande, plafonnée par la capacité de calcul.
    pub fn effective_speed(&self) -> f64 {
        self.speed.years_per_second() * self.capacity_ratio()
    }

    fn capacity_ratio(&self) -> f64 {
        match self.cost_s {
            Some(cost) if cost > 0.0 => ((1.0 / cost) / self.tps).min(1.0),
            _ => 1.0,
        }
    }

    fn period_s(&self) -> f64 {
        1.0 / self.tps
    }
}

fn clamp_rate(tps: f64) -> f64 {
    if tps.is_finite() {
        tps.clamp(MIN_TPS, MAX_TPS)
    } else {
        TARGET_TPS
    }
}

fn is_duration(seconds: f64) -> bool {
    seconds.is_finite() && seconds >= 0.0
}
