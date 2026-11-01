//! Traduction des structures du monde en JSON pour l'interface (formats de REGLES §13 et `region.ts`).

use crate::world::{CellDetail, Vitals, WorldEvent, WorldInfo, LOOKUP_HEIGHT, LOOKUP_WIDTH};
use serde_json::{json, Value};

pub fn world_info(info: &WorldInfo) -> Value {
    json!({
        "cells": info.cells,
        "seed": info.seed,
        "name": info.name,
        "lookupWidth": LOOKUP_WIDTH,
        "lookupHeight": LOOKUP_HEIGHT,
    })
}

/// Valeurs globales : O₂ et CO₂ (matière), espèces et biomasse (vivant), température moyenne (climat).
pub fn vitals(vitals: &Vitals) -> Value {
    json!({
        "o2Percent": vitals.o2_percent,
        "co2Ppm": vitals.co2_ppm,
        "meanTempC": vitals.mean_temp_c,
        "species": vitals.species,
        "biomassGtC": vitals.biomass_gtc,
        "era": vitals.era,
    })
}

pub fn cell(detail: &CellDetail) -> Value {
    let species: Vec<Value> = detail
        .species
        .iter()
        .map(|line| {
            json!({
                "id": line.id,
                "name": line.name,
                "role": line.role.as_str(),
                "population": line.population,
                "trend": [],
            })
        })
        .collect();
    json!({
        "lat": detail.lat_deg,
        "lon": detail.lon_deg,
        "elevation": detail.elevation_m,
        "temperature": detail.temperature_c,
        "precipitation": detail.precip_mm_yr,
        "iceFraction": detail.ice_fraction,
        "vegCover": detail.veg_cover,
        "nppGcM2Yr": detail.npp_gc_m2_yr,
        "soil": {
            "water": detail.soil.water,
            "carbon": detail.soil.carbon,
            "nitrogen": detail.soil.nitrogen,
            "phosphorus": detail.soil.phosphorus,
        },
        "biome": detail.biome,
        "speciesCount": detail.species_count,
        "species": species,
    })
}

pub fn events(events: &[WorldEvent]) -> Value {
    Value::Array(events.iter().map(event).collect())
}

fn event(event: &WorldEvent) -> Value {
    json!({
        "tick": event.tick,
        "years": event.years,
        "kind": event.kind,
        "title": event.title,
        "body": event.body,
        "cell": event.cell,
    })
}
