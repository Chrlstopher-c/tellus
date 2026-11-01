//! Pas de temps de la matière (contrat REGLES § 12) : sorties du début de tick, sous-pas biologiques, transport,
//! géologie, équilibre océan-atmosphère, rapport. Le graine et le tick ne servent à rien : la matière est déterministe.

use super::carbon;
use super::error::MatterError;
use super::forcing::{Forcing, Scratch};
use super::geology;
use super::inputs::MatterInputs;
use super::nitrogen;
use super::params::BIO_SUBSTEPS;
use super::phosphorus;
use super::report::{MatterReport, TickStart};
use super::state::MatterState;
use super::water;
use crate::clock::Regime;
use crate::shared::grid::Grid;

/// Avance la matière d'un tick de `dt_years` ans. `regime` choisit le nombre de sous-pas biologiques (12 pour
/// Δt ≤ 1 an, sinon relaxation exacte en un pas).
pub fn step(
    state: &mut MatterState,
    grid: &Grid,
    inputs: &MatterInputs<'_>,
    dt_years: f64,
    regime: Regime,
    tick: u64,
    _seed: u64,
) -> Result<MatterReport, MatterError> {
    if !dt_years.is_finite() || dt_years <= 0.0 {
        return Err(MatterError::InvalidTimeStep(dt_years));
    }
    let cells = state.cell_count();
    if grid.cell_count() != cells {
        return Err(MatterError::LengthMismatch {
            field: "grid",
            expected: cells,
            got: grid.cell_count(),
        });
    }
    inputs.validate(cells)?;
    if inputs.is_ocean != state.geometry.is_ocean.as_slice() {
        return Err(MatterError::MaskChanged);
    }
    let start = TickStart::capture(state);
    let forcing = Forcing::compute(state, inputs);
    publish_outputs(state, &forcing);
    let sub_steps = match regime {
        Regime::Fine | Regime::Annual => BIO_SUBSTEPS,
        Regime::Secular | Regime::Geological | Regime::Frozen => 1,
    };
    let sub_dt = dt_years / sub_steps as f64;
    phosphorus::weather(&mut state.phosphorus, &state.geometry, forcing.weathering, dt_years);
    let mut scratch = Scratch::new(cells);
    for _ in 0..sub_steps {
        biology_substep(state, inputs, &forcing, &mut scratch, sub_dt);
    }
    transport(state, &forcing, dt_years);
    let MatterState {
        geometry,
        carbon,
        oxygen,
        volcanic_factor,
        ..
    } = state;
    geology::super_tick(carbon, oxygen, &forcing, dt_years, *volcanic_factor);
    carbon::end_of_tick(carbon, geometry, inputs.temperature_c, dt_years);
    Ok(MatterReport::new(state, &start, &forcing, tick, regime))
}

/// Copie les sorties par cellule (début de tick) dans l'état, en `f32` comme le contrat l'exige.
fn publish_outputs(state: &mut MatterState, forcing: &Forcing) {
    copy_f32(&mut state.npp_cap_gc_m2_yr, &forcing.npp_cap);
    copy_f32(&mut state.n_avail, &forcing.n_avail);
    copy_f32(&mut state.p_avail, &forcing.p_avail);
    copy_f32(&mut state.nutrient_ocean, &forcing.nutrient_ocean);
}

fn copy_f32(destination: &mut [f32], source: &[f64]) {
    for (dst, &src) in destination.iter_mut().zip(source) {
        *dst = src as f32;
    }
}

fn biology_substep(
    state: &mut MatterState,
    inputs: &MatterInputs<'_>,
    forcing: &Forcing,
    scratch: &mut Scratch,
    dt: f64,
) {
    let MatterState {
        geometry,
        carbon,
        nitrogen,
        phosphorus,
        water,
        ..
    } = state;
    water::substep(water, geometry, inputs, dt, scratch);
    carbon::land_substep(carbon, geometry, forcing, scratch, dt);
    nitrogen::land_substep(nitrogen, geometry, forcing, scratch, dt);
    phosphorus::land_substep(phosphorus, geometry, forcing, scratch, dt);
}

/// Fin de biologie : remontée et rivières d'abord (renouvellement de la surface océanique), puis export marin
/// linéaire appliqué une fois par tick (équivalent exact à 12 sous-pas, sans vider la surface avant sa recharge).
fn transport(state: &mut MatterState, forcing: &Forcing, dt: f64) {
    let MatterState {
        geometry,
        carbon,
        nitrogen,
        phosphorus,
        ..
    } = state;
    nitrogen::transport(nitrogen, geometry, dt);
    phosphorus::transport(phosphorus, geometry, dt);
    carbon::marine_export(carbon, geometry, forcing, dt);
    nitrogen::marine_export(nitrogen, geometry, forcing, dt);
    phosphorus::marine_export(phosphorus, geometry, forcing, dt);
}
