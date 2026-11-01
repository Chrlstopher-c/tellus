//! G2 : mouvement de chaque plaque par rotation d'Euler. Pôle unitaire u, vitesse angulaire ω (rad/s) ;
//! vitesse de surface v(x) = ωR·(u × x̂), tangente à la sphère.

use std::f64::consts::TAU;

use super::vec3::{cross, scale, Vec3};
use crate::shared::rng::unit_f64;
use crate::shared::units::SECONDS_PER_YEAR;

pub(super) const STEP_ID: u64 = 2;
const SPEED_MIN_CM_YR: f64 = 1.0;
const SPEED_MAX_CM_YR: f64 = 15.0;
const STREAM_AXIS_Z: u32 = 0;
const STREAM_AXIS_PHI: u32 = 1;
const STREAM_SPEED: u32 = 2;

/// Plaque : pôle d'Euler unitaire, vitesse angulaire (rad/s) et type de croûte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plate {
    pub pole: [f64; 3],
    pub omega_rad_s: f64,
    pub continental: bool,
}

impl Plate {
    /// Vitesse équatoriale équivalente ωR, en cm/an.
    pub fn speed_cm_per_year(&self, radius_m: f64) -> f64 {
        self.omega_rad_s * radius_m * 100.0 * SECONDS_PER_YEAR
    }
}

/// Un pôle par plaque, uniforme sur la sphère ; vitesse équatoriale tirée dans [1, 15] cm/an.
pub(super) fn euler_plates(seed: u64, continental: &[bool], radius_m: f64) -> Vec<Plate> {
    continental
        .iter()
        .enumerate()
        .map(|(p, &is_continental)| {
            let index = p as u32;
            let speed_cm_yr =
                SPEED_MIN_CM_YR + (SPEED_MAX_CM_YR - SPEED_MIN_CM_YR) * unit_f64(seed, STEP_ID, index, STREAM_SPEED);
            Plate {
                pole: random_unit(seed, index),
                omega_rad_s: speed_cm_yr / 100.0 / SECONDS_PER_YEAR / radius_m,
                continental: is_continental,
            }
        })
        .collect()
}

fn random_unit(seed: u64, index: u32) -> Vec3 {
    let z = 2.0 * unit_f64(seed, STEP_ID, index, STREAM_AXIS_Z) - 1.0;
    let phi = TAU * unit_f64(seed, STEP_ID, index, STREAM_AXIS_PHI);
    let ring = (1.0 - z * z).sqrt();
    [ring * phi.cos(), ring * phi.sin(), z]
}

/// Vitesse de surface (m/s) d'une plaque au point unitaire `x`.
pub(super) fn surface_velocity(plate: &Plate, x: Vec3, radius_m: f64) -> Vec3 {
    scale(cross(plate.pole, x), plate.omega_rad_s * radius_m)
}
