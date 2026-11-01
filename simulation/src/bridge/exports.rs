//! Exports napi consommés par le processus utilityProcess (REGLES §13). Chaque export délègue au monde courant ;
//! aucun n'écrit de règle : création, avancée, lecture et traduction.

use super::encode;
use super::slot::WorldSlot;
use crate::world::{World, WorldError, WorldRequest};
use napi::bindgen_prelude::Float32Array;
use napi_derive::napi;

static WORLD: WorldSlot = WorldSlot::new();

#[napi]
pub fn create_world(config_json: String) -> napi::Result<String> {
    let request = WorldRequest::from_json(&config_json).map_err(to_js_error)?;
    let world = World::create(&request).map_err(to_js_error)?;
    let info = encode::world_info(world.info()).to_string();
    WORLD.replace(world).map_err(to_js_error)?;
    Ok(info)
}

#[napi]
pub fn lookup_ids() -> napi::Result<Float32Array> {
    WORLD
        .with(|world| Ok(Float32Array::new(world.lookup().ids.clone())))
        .map_err(to_js_error)
}

#[napi]
pub fn lookup_weights() -> napi::Result<Float32Array> {
    WORLD
        .with(|world| Ok(Float32Array::new(world.lookup().weights.clone())))
        .map_err(to_js_error)
}

#[napi]
pub fn schedule(wall_seconds: f64, years_per_second: f64, last_tick_seconds: f64) -> napi::Result<f64> {
    WORLD
        .with_mut(|world| world.schedule(wall_seconds, years_per_second, last_tick_seconds))
        .map_err(to_js_error)
}

#[napi]
pub fn step(dt_years: f64) -> napi::Result<f64> {
    WORLD.with_mut(|world| world.step(dt_years)).map_err(to_js_error)
}

#[napi]
pub fn snapshot() -> napi::Result<Float32Array> {
    WORLD
        .with(|world| Ok(Float32Array::new(world.snapshot())))
        .map_err(to_js_error)
}

#[napi]
pub fn vitals() -> napi::Result<String> {
    WORLD
        .with(|world| Ok(encode::vitals(&world.vitals()).to_string()))
        .map_err(to_js_error)
}

#[napi]
pub fn inspect(cell: u32) -> napi::Result<String> {
    WORLD
        .with(|world| world.inspect(cell).map(|detail| encode::cell(&detail).to_string()))
        .map_err(to_js_error)
}

/// Les pouvoirs du joueur ne sont pas encore branchés : toute commande est refusée explicitement.
#[napi]
pub fn intervene(_command_json: String) -> napi::Result<String> {
    Err(to_js_error(WorldError::PowersUnavailable))
}

#[napi]
pub fn drain_events() -> napi::Result<String> {
    WORLD
        .with_mut(|world| Ok(encode::events(&world.drain_events()).to_string()))
        .map_err(to_js_error)
}

fn to_js_error(error: WorldError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}
