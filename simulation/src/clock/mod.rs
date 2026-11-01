//! Horloge multi-échelle : vitesses, régimes par système, ordonnancement des ticks (docs/REGLES.md § 1).

mod regime;
mod scheduler;
mod speed;
#[cfg(test)]
mod tests;

pub use regime::{regime, Regime, System};
pub use scheduler::{Scheduler, Tick};
pub use speed::Speed;
