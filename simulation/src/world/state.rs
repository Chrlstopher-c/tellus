//! Monde vivant : possède la grille, la planète, le climat, la matière, l'horloge, la table de rendu et le journal.
//! `step` appelle les domaines dans l'ordre de REGLES §12 (climat → matière) avec le régime de chaque système.

use super::error::WorldError;
use super::journal::{Journal, WorldEvent};
use super::lookup::Lookup;
use super::schedule::speed_for_rate;
use crate::climate::{self, ClimateInputs, ClimateState};
use crate::clock::{self, Scheduler, System};
use crate::genesis::Planet;
use crate::matter::{self, MatterInputs, MatterState, SoilView};
use crate::shared::grid::Grid;

/// Identité du monde, fixée à la création.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldInfo {
    pub name: String,
    pub seed: u64,
    pub cells: usize,
}

/// Indicateurs globaux affichés en permanence.
#[derive(Debug, Clone, PartialEq)]
pub struct Vitals {
    pub o2_percent: f64,
    pub co2_ppm: f64,
    pub mean_temp_c: f64,
    pub era: String,
}

/// Détail d'une cellule pour l'inspecteur (champs absents du domaine : 0 ou vides côté pont).
#[derive(Debug, Clone, PartialEq)]
pub struct CellDetail {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub elevation_m: f64,
    pub temperature_c: f64,
    pub ice_fraction: f64,
    pub soil: SoilView,
}

/// Champs climatiques convertis en f32 pour la matière (contrat §12 : tranches par cellule).
pub(super) struct MatterFeed {
    pub(super) temperature: Vec<f32>,
    pub(super) precip: Vec<f32>,
    pub(super) insolation: Vec<f32>,
    pub(super) ice: Vec<f32>,
}

impl MatterFeed {
    pub(super) fn from_climate(climate: &ClimateState) -> Self {
        Self {
            temperature: to_f32(&climate.temperature_c),
            precip: to_f32(&climate.precip_mm_yr),
            insolation: to_f32(&climate.insolation_w_m2),
            ice: to_f32(&climate.ice_fraction),
        }
    }
}

pub struct World {
    pub(super) info: WorldInfo,
    pub(super) grid: Grid,
    pub(super) planet: Planet,
    pub(super) climate: ClimateState,
    pub(super) matter: MatterState,
    pub(super) clock: Scheduler,
    pub(super) lookup: Lookup,
    pub(super) journal: Journal,
    pub(super) is_ocean: Vec<bool>,
    /// Zéros par cellule pour les entrées de domaines pas encore branchés (hydrologie, vivant).
    pub(super) zeros: Vec<f32>,
    pub(super) time_years: f64,
    pub(super) ticks: u64,
}

impl World {
    pub fn info(&self) -> &WorldInfo {
        &self.info
    }

    pub fn lookup(&self) -> &Lookup {
        &self.lookup
    }

    /// Temps simulé total, années.
    pub fn time_years(&self) -> f64 {
        self.time_years
    }

    /// Durée du prochain tick pour une vitesse (années par seconde murale), après le temps mural écoulé et le coût du
    /// tick précédent. Renvoie 0 quand aucun tick n'est dû. Le coût est enregistré avant le temps mural : le calcul
    /// tient dans l'intervalle qui le contient.
    pub fn schedule(&mut self, wall_s: f64, years_per_second: f64, last_tick_s: f64) -> Result<f64, WorldError> {
        self.clock.set_speed(speed_for_rate(years_per_second)?);
        self.clock.record_cost(last_tick_s);
        self.clock.accrue(wall_s);
        Ok(self.clock.next_tick().map_or(0.0, |tick| tick.dt_years))
    }

    /// Avance le monde de `dt_years` (0 = aucun changement). Renvoie le temps simulé total.
    pub fn step(&mut self, dt_years: f64) -> Result<f64, WorldError> {
        if dt_years == 0.0 {
            return Ok(self.time_years);
        }
        if !(dt_years.is_finite() && dt_years > 0.0) {
            return Err(WorldError::InvalidDuration(dt_years));
        }
        self.climate_step(dt_years)?;
        self.matter_step(dt_years)?;
        self.time_years += dt_years;
        self.ticks += 1;
        Ok(self.time_years)
    }

    /// Un instantané par cellule, 4 flottants : altitude m, température °C, précipitations mm/an, couvert végétal.
    pub fn snapshot(&self) -> Vec<f32> {
        let mut fields = Vec::with_capacity(self.planet.elevation_m.len() * 4);
        for (cell, &elevation) in self.planet.elevation_m.iter().enumerate() {
            fields.push(elevation);
            fields.push(self.climate.temperature_c[cell] as f32);
            fields.push(self.climate.precip_mm_yr[cell] as f32);
            fields.push(0.0);
        }
        fields
    }

    pub fn vitals(&self) -> Vitals {
        let mean_temp_c = area_mean(&self.grid, &self.climate.temperature_c);
        Vitals {
            o2_percent: self.matter.o2_fraction() * 100.0,
            co2_ppm: self.matter.co2_ppm(),
            mean_temp_c,
            era: era_label(mean_temp_c),
        }
    }

    pub fn inspect(&self, cell: u32) -> Result<CellDetail, WorldError> {
        let index = cell as usize;
        let cells = self.grid.cell_count();
        if index >= cells {
            return Err(WorldError::CellOutOfRange { cell, cells });
        }
        let [x, y, z] = self.grid.centers()[index];
        Ok(CellDetail {
            lat_deg: y.atan2(x.hypot(z)).to_degrees(),
            lon_deg: x.atan2(z).to_degrees(),
            elevation_m: f64::from(self.planet.elevation_m[index]),
            temperature_c: self.climate.temperature_c[index],
            ice_fraction: self.climate.ice_fraction[index],
            soil: self.matter.soil(index),
        })
    }

    /// Événements enregistrés depuis le dernier appel.
    pub fn drain_events(&mut self) -> Vec<WorldEvent> {
        self.journal.drain()
    }

    fn climate_step(&mut self, dt_years: f64) -> Result<(), WorldError> {
        let regime = clock::regime(System::Climate, dt_years);
        let inputs = ClimateInputs {
            co2_ppm: self.matter.co2_ppm(),
            surface_albedo: None,
        };
        climate::step(
            &mut self.climate,
            &self.grid,
            &inputs,
            dt_years,
            regime,
            self.ticks,
            self.info.seed,
        )?;
        Ok(())
    }

    fn matter_step(&mut self, dt_years: f64) -> Result<(), WorldError> {
        let regime = clock::regime(System::Matter, dt_years);
        let feed = MatterFeed::from_climate(&self.climate);
        let inputs = MatterInputs {
            temperature_c: &feed.temperature,
            precip_mm_yr: &feed.precip,
            insolation_w_m2: &feed.insolation,
            ice_fraction: &feed.ice,
            is_ocean: &self.is_ocean,
            elevation_m: &self.planet.elevation_m,
            npp_realized_gc_m2_yr: &self.zeros,
            biomass_gc_m2: &self.zeros,
        };
        matter::step(
            &mut self.matter,
            &self.grid,
            &inputs,
            dt_years,
            regime,
            self.ticks,
            self.info.seed,
        )?;
        Ok(())
    }
}

fn to_f32(values: &[f64]) -> Vec<f32> {
    values.iter().map(|&v| v as f32).collect()
}

fn area_mean(grid: &Grid, values: &[f64]) -> f64 {
    let areas = grid.areas_m2();
    let total: f64 = areas.iter().sum();
    areas.iter().zip(values).map(|(area, value)| area * value).sum::<f64>() / total
}

/// Qualificatif de l'époque, dérivé de la température moyenne ; pas de vie ni d'oxygène au MVP.
fn era_label(mean_temp_c: f64) -> String {
    let climate = if mean_temp_c < -10.0 {
        "gelée"
    } else if mean_temp_c > 40.0 {
        "brûlante"
    } else {
        "jeune"
    };
    format!("Âge du monde · planète {climate}, sans vie")
}
