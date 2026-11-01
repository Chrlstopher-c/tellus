//! Exports napi consommés par le processus utilityProcess. Chaque export délègue à `WorldSlot`.

use super::world::{WorldError, WorldSlot};
use napi::bindgen_prelude::Float32Array;
use napi_derive::napi;

static WORLD: WorldSlot = WorldSlot::new();

#[napi]
pub fn create_world(seed: u32, level: u32) -> napi::Result<u32> {
    let cells = WORLD.create(seed, level).map_err(to_js_error)?;
    u32::try_from(cells).map_err(|_| napi::Error::from_reason(format!("{cells} cellules hors de u32")))
}

#[napi]
pub fn step(dt_years: f64) -> napi::Result<f64> {
    WORLD.step(dt_years).map_err(to_js_error)
}

#[napi]
pub fn snapshot_elevation() -> napi::Result<Float32Array> {
    WORLD.snapshot().map(Float32Array::new).map_err(to_js_error)
}

fn to_js_error(error: WorldError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}
