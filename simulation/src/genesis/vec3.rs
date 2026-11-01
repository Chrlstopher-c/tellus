//! Algèbre vectorielle minimale sur `[f64; 3]`, réservée à la genèse.

pub(super) type Vec3 = [f64; 3];

pub(super) fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(super) fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub(super) fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

pub(super) fn normalize(a: Vec3) -> Vec3 {
    scale(a, 1.0 / norm(a))
}

/// Angle entre deux vecteurs, en radians (stable près de 0 et de π).
pub(super) fn angle(a: Vec3, b: Vec3) -> f64 {
    norm(cross(a, b)).atan2(dot(a, b))
}
