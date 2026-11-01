//! Correspondance entre les vitesses de jeu envoyées par l'interface (années simulées par seconde murale)
//! et `clock::Speed`, pour que `clock::Scheduler` produise les ticks.

use super::error::WorldError;
use crate::clock::Speed;

const SPEEDS: [Speed; 7] = [
    Speed::Pause,
    Speed::Day,
    Speed::Year,
    Speed::Century,
    Speed::TenKyr,
    Speed::Myr,
    Speed::TenMyr,
];
const RATE_TOLERANCE: f64 = 1e-9;

pub fn speed_for_rate(years_per_second: f64) -> Result<Speed, WorldError> {
    if !years_per_second.is_finite() || years_per_second < 0.0 {
        return Err(WorldError::InvalidRate(years_per_second));
    }
    SPEEDS
        .into_iter()
        .find(|speed| {
            let expected = speed.years_per_second();
            (expected - years_per_second).abs() <= RATE_TOLERANCE * expected.max(1.0)
        })
        .ok_or(WorldError::InvalidRate(years_per_second))
}
