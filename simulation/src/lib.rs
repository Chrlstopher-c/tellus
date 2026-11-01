//! Noyau de simulation Tellus : vérité du monde, sans notion d'affichage.

#[cfg(feature = "node")]
pub mod bridge;
pub mod climate;
pub mod clock;
pub mod genesis;
pub mod matter;
pub mod shared;
pub mod world;
