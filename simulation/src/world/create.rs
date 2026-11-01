//! Construction d'un monde : genèse ou corps réel, puis état initial de chaque domaine (climat, matière).

use super::error::WorldError;
use super::journal::{Journal, WorldEvent};
use super::lookup;
use super::name::name_from_seed;
use super::physics::{generated_body, playable_body_id, GeneratedPhysics};
use super::request::{GeneratedSpec, Source, WorldRequest, MAX_LEVEL};
use super::state::{ClimateFeed, World, WorldInfo};
use crate::climate::body::Body;
use crate::climate::{self, ClimateConfig, OlrLaw, Surface};
use crate::clock::{Scheduler, Speed};
use crate::genesis::{self, Planet, WorldParams};
use crate::life;
use crate::matter::{self, MatterConfig, MatterInputs};
use crate::shared::grid::Grid;

/// Pression de surface d'un monde généré, bar (atmosphère de type terrestre).
const GENERATED_PRESSURE_BAR: f64 = 1.0;
/// CO₂ de référence de la loi d'effet de serre d'un monde généré, ppm.
const GENERATED_CO2_REF_PPM: f64 = 280.0;

/// Tout ce qui décrit un monde avant l'assemblage des domaines.
struct Parts {
    name: String,
    seed: u64,
    grid: Grid,
    planet: Planet,
    body: Body,
    climate_config: ClimateConfig,
    matter_config: MatterConfig,
    description: String,
}

impl World {
    /// Crée un monde depuis une requête validée. Coûteux (genèse, table de rendu) : à appeler hors de la boucle.
    pub fn create(request: &WorldRequest) -> Result<Self, WorldError> {
        let level = check_level(request.level)?;
        let parts = match &request.source {
            Source::Generated(spec) => generated(spec, level)?,
            Source::Body(name) => real_body(name, level)?,
        };
        assemble(parts)
    }
}

fn check_level(level: u32) -> Result<u8, WorldError> {
    if level > MAX_LEVEL {
        return Err(WorldError::InvalidLevel(level));
    }
    u8::try_from(level).map_err(|_| WorldError::InvalidLevel(level))
}

fn generated(spec: &GeneratedSpec, level: u8) -> Result<Parts, WorldError> {
    let physics = GeneratedPhysics::from_spec(spec);
    let name = name_from_seed(spec.seed);
    let grid = Grid::new(level, physics.radius_m);
    let params = WorldParams {
        seed: spec.seed,
        level,
        radius_m: physics.radius_m,
        water_fraction: physics.water_fraction,
    };
    let planet = genesis::generate(&grid, &params)?;
    let description = format!(
        "Rayon {:.0} km · gravité {:.2} m/s² · {:.0} % d'eau",
        physics.radius_m / 1_000.0,
        physics.gravity_ms2,
        spec.water_percent,
    )
    .replace('.', ",");
    Ok(Parts {
        body: generated_body(&name, &physics)?,
        climate_config: ClimateConfig::for_world(GENERATED_PRESSURE_BAR, GENERATED_CO2_REF_PPM),
        matter_config: MatterConfig::default(),
        name,
        seed: spec.seed,
        grid,
        planet,
        description,
    })
}

fn real_body(name: &str, level: u8) -> Result<Parts, WorldError> {
    let id = playable_body_id(name)?;
    let body = climate::body::body(id)?;
    let grid = Grid::new(level, body.radius_km * 1_000.0);
    let planet = genesis::real_body(&grid, id)?;
    let climate_config = ClimateConfig::for_body(&body)?;
    let matter_config = matter_start(&climate_config);
    Ok(Parts {
        name: body.name_fr.clone(),
        seed: 0,
        grid,
        planet,
        description: format!("Corps réel · {} · données NASA", body.name_fr),
        body,
        climate_config,
        matter_config,
    })
}

/// CO₂ de départ de la matière : planète jeune à CO₂ d'inventaire (REGLES §6) pour la Terre ; autres corps : CO₂
/// de référence de leur atmosphère réelle, pour que la calibration de l'effet de serre reste valable.
fn matter_start(climate_config: &ClimateConfig) -> MatterConfig {
    match climate_config.law {
        OlrLaw::Earth => MatterConfig::default(),
        OlrLaw::Grey { co2_ref_ppm, .. } => MatterConfig {
            co2_ppm: co2_ref_ppm,
            ..MatterConfig::default()
        },
    }
}

fn assemble(parts: Parts) -> Result<World, WorldError> {
    let Parts {
        name,
        seed,
        grid,
        planet,
        body,
        climate_config,
        matter_config,
        description,
    } = parts;
    let elevation: Vec<f64> = planet.elevation_m.iter().map(|&h| f64::from(h)).collect();
    // Océan = sous le niveau de la mer, sur un monde qui a de l'eau (un corps sec n'a aucune croûte « océan »).
    let has_water = planet.crust.iter().any(|&c| c < 0.5);
    let is_ocean: Vec<bool> = planet.elevation_m.iter().map(|&h| has_water && h < 0.0).collect();
    let surface = Surface {
        elevation_m: &elevation,
        is_ocean: &is_ocean,
    };
    let climate = climate::init(&grid, &body, &surface, &climate_config)?;
    let feed = ClimateFeed::from_climate(&climate);
    let zeros = vec![0.0_f32; grid.cell_count()];
    let inputs = MatterInputs {
        temperature_c: &feed.temperature,
        precip_mm_yr: &feed.precip,
        insolation_w_m2: &feed.insolation,
        ice_fraction: &feed.ice,
        is_ocean: &is_ocean,
        elevation_m: &planet.elevation_m,
        npp_realized_gc_m2_yr: &zeros,
        biomass_gc_m2: &zeros,
    };
    let matter = matter::init(&matter_config, &grid, &inputs)?;
    let life = life::init(&grid);
    let lookup = lookup::build(&grid);
    let mut journal = Journal::default();
    journal.record(formation_event(description));
    let cells = grid.cell_count();
    Ok(World {
        info: WorldInfo { name, seed, cells },
        grid,
        planet,
        climate,
        matter,
        life,
        clock: Scheduler::new(Speed::Pause),
        lookup,
        journal,
        is_ocean,
        zeros,
        time_years: 0.0,
        ticks: 0,
    })
}

fn formation_event(description: String) -> WorldEvent {
    WorldEvent {
        tick: 0,
        years: 0.0,
        kind: "geology",
        title: "Formation du monde".to_owned(),
        body: description,
        cell: None,
    }
}
