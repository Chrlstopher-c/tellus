//! Pas implicite de l'EBM 2D Budyko–Sellers. Bilan linéarisé autour de T^n : OLR(T) ≈ OLR(T^n) + λ(T − T^n),
//! albédo et capacité évalués en T^n (explicites), diffusion et pente de l'OLR implicites :
//! `(C/Δt + λ)·T^{n+1} − D·∇²T^{n+1} = (C/Δt + λ)·T^n + Q(1 − α) − OLR(T^n)`.
//! Le solveur part de T^n (départ à chaud). Bornage de la température et diagnostics du pas ensuite.

use super::error::ClimateError;
use super::greenhouse::{co2_forcing, grey_olr, grey_olr_slope, scaled_tau, EARTH_OLR_A_W_M2, EARTH_OLR_B_W_M2_K};
use super::ice::{ice_fraction, surface_albedo};
use super::solver::{solve, ImplicitOperator};
use super::state::{ClimateModel, ClimateState, OlrLaw, TEMP_MAX_C, TEMP_MIN_C};

const KELVIN_OFFSET: f64 = 273.15;

/// Forçages d'un pas : CO₂ (global), durée en secondes, insolation et albédo libre par cellule.
pub(super) struct Forcing<'a> {
    pub co2_ppm: f64,
    pub dt_s: f64,
    pub insolation: &'a [f64],
    pub free_albedo: &'a [f64],
}

/// Diagnostics d'un pas implicite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SubstepReport {
    pub max_change_k: f64,
    pub cg_iterations: u32,
    pub relative_residual: f64,
    pub energy_error: f64,
    pub clamped: u32,
    pub cap_reached: bool,
}

/// Avance `state` d'un pas implicite.
pub(super) fn substep(
    model: &ClimateModel,
    state: &mut ClimateState,
    forcing: &Forcing<'_>,
) -> Result<SubstepReport, ClimateError> {
    let surface = surface_c(model, &state.energy_c);
    let ice = ice_fractions(model, &surface);
    let albedo: Vec<f64> = ice
        .iter()
        .zip(forcing.free_albedo)
        .map(|(&f, &free)| surface_albedo(f, free))
        .collect();
    let capacity = blended_capacity(model, &ice);
    let (olr, slope) = outgoing(model, &state.energy_c, forcing.co2_ppm);
    let mass: Vec<f64> = capacity.iter().zip(&slope).map(|(c, s)| c / forcing.dt_s + s).collect();
    let rhs: Vec<f64> = (0..olr.len())
        .map(|i| {
            let absorbed = forcing.insolation[i] * (1.0 - albedo[i]);
            model.mesh.area_sr()[i] * (mass[i] * state.energy_c[i] + absorbed - olr[i])
        })
        .collect();
    let op = ImplicitOperator::new(&model.mesh, &mass, model.diffusivity);
    let mut next = state.energy_c.clone();
    let solved = solve(&op, &rhs, &mut next);
    let (max_change_k, clamped) = bound_change(&state.energy_c, &mut next)?;
    state.energy_c = next;
    state.absorbed_w_m2 = forcing
        .insolation
        .iter()
        .zip(&albedo)
        .map(|(q, a)| q * (1.0 - a))
        .collect();
    state.olr_w_m2 = olr;
    diagnose(model, state);
    Ok(SubstepReport {
        max_change_k,
        cg_iterations: solved.iterations,
        relative_residual: solved.relative_residual,
        energy_error: solved.energy_error,
        clamped,
        cap_reached: solved.cap_reached,
    })
}

/// Température de surface (°C) : température d'énergie moins le gradient d'altitude sur les terres.
pub(super) fn surface_c(model: &ClimateModel, energy_c: &[f64]) -> Vec<f64> {
    energy_c.iter().zip(&model.lapse_offset_k).map(|(t, z)| t - z).collect()
}

/// Remplit les diagnostics de sortie (température de surface, glace) à partir de l'état courant.
pub(super) fn diagnose(model: &ClimateModel, state: &mut ClimateState) {
    let surface = surface_c(model, &state.energy_c);
    state.ice_fraction = ice_fractions(model, &surface);
    state.temperature_c = surface;
}

fn ice_fractions(model: &ClimateModel, surface: &[f64]) -> Vec<f64> {
    if model.ice_feedback {
        surface.iter().map(|&t| ice_fraction(t)).collect()
    } else {
        vec![0.0; surface.len()]
    }
}

/// Capacité par surface : mélange océan/terre et glace selon la fraction de glace.
fn blended_capacity(model: &ClimateModel, ice: &[f64]) -> Vec<f64> {
    ice.iter()
        .zip(&model.surface_capacity)
        .map(|(&f, &c)| (1.0 - f) * c + f * model.ice_capacity)
        .collect()
}

/// OLR au sommet (W m⁻²) et sa pente par rapport à la température d'énergie (W m⁻² K⁻¹).
fn outgoing(model: &ClimateModel, energy_c: &[f64], co2_ppm: f64) -> (Vec<f64>, Vec<f64>) {
    match model.law {
        OlrLaw::Earth => {
            let forcing = co2_forcing(co2_ppm);
            let olr = energy_c
                .iter()
                .map(|t| EARTH_OLR_A_W_M2 + EARTH_OLR_B_W_M2_K * t - forcing)
                .collect();
            (olr, vec![EARTH_OLR_B_W_M2_K; energy_c.len()])
        }
        OlrLaw::Grey { tau_ref, co2_ref_ppm } => {
            let tau = scaled_tau(tau_ref, co2_ppm, co2_ref_ppm);
            let olr = energy_c.iter().map(|t| grey_olr(t + KELVIN_OFFSET, tau)).collect();
            let slope = energy_c
                .iter()
                .map(|t| grey_olr_slope(t + KELVIN_OFFSET, tau))
                .collect();
            (olr, slope)
        }
    }
}

/// Borne la solution du solveur ; renvoie |ΔT| max et le nombre de bornes atteintes. Un NaN ou une infinité est une
/// erreur, pas un clamp.
fn bound_change(previous: &[f64], next: &mut [f64]) -> Result<(f64, u32), ClimateError> {
    let mut max_change: f64 = 0.0;
    let mut clamped = 0;
    for (cell, (before, after)) in previous.iter().zip(next.iter_mut()).enumerate() {
        if !after.is_finite() {
            return Err(ClimateError::NonFinite {
                field: "température",
                cell,
            });
        }
        let bounded = after.clamp(TEMP_MIN_C, TEMP_MAX_C);
        if bounded != *after {
            clamped += 1;
        }
        max_change = max_change.max((bounded - before).abs());
        *after = bounded;
    }
    Ok((max_change, clamped))
}
