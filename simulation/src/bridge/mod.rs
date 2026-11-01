//! Pont napi vers le processus utilityProcess : exports et état du monde tenu par le processus.
//! Aucune règle du monde ici, seulement la traduction des commandes et des instantanés.

mod exports;
mod world;

#[cfg(test)]
mod tests;

pub use exports::{create_world, snapshot_elevation, step};
pub use world::{World, WorldError, WorldSlot, MAX_LEVEL};
