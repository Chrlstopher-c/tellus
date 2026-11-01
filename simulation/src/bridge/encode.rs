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

/// Champs sans source au MVP (o₂ et espèces : vivant absent) envoyés à zéro.
pub fn vitals(vitals: &Vitals) -> Value {
    json!({
        "o2Percent": vitals.o2_percent,
        "co2Ppm": vitals.co2_ppm,
        "meanTempC": vitals.mean_temp_c,
        "species": 0,
        "biomassGtC": 0,
        "era": vitals.era,
    })
}

pub fn cell(detail: &CellDetail) -> Value {
    json!({
        "lat": detail.lat_deg,
        "lon": detail.lon_deg,
        "elevation": detail.elevation_m,
        "temperature": detail.temperature_c,
        "precipitation": 0,
        "iceFraction": detail.ice_fraction,
        "vegCover": 0,
        "nppGcM2Yr": 0,
        "soil": {
            "water": detail.soil.water,
            "carbon": detail.soil.carbon,
            "nitrogen": detail.soil.nitrogen,
            "phosphorus": detail.soil.phosphorus,
        },
        "biome": null,
        "speciesCount": 0,
        "species": [],
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
