//! Emplacement unique du monde courant, protégé par un verrou. Le pont n'y touche que par `with` / `with_mut`.

use crate::world::{World, WorldError};
use std::sync::{Mutex, MutexGuard};

pub struct WorldSlot {
    inner: Mutex<Option<World>>,
}

impl WorldSlot {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Remplace le monde courant ; l'ancien est détruit.
    pub fn replace(&self, world: World) -> Result<(), WorldError> {
        *self.lock()? = Some(world);
        Ok(())
    }

    pub fn with<T>(&self, read: impl FnOnce(&World) -> Result<T, WorldError>) -> Result<T, WorldError> {
        let guard = self.lock()?;
        read(guard.as_ref().ok_or(WorldError::NoWorld)?)
    }

    pub fn with_mut<T>(&self, write: impl FnOnce(&mut World) -> Result<T, WorldError>) -> Result<T, WorldError> {
        let mut guard = self.lock()?;
        write(guard.as_mut().ok_or(WorldError::NoWorld)?)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<World>>, WorldError> {
        self.inner.lock().map_err(|_| WorldError::Poisoned)
    }
}

impl Default for WorldSlot {
    fn default() -> Self {
        Self::new()
    }
}
