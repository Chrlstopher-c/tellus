//! Algèbre vectorielle minimale sur `[f64; 3]`.

pub type Vec3 = [f64; 3];

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

pub fn normalize(a: Vec3) -> Vec3 {
    scale(a, 1.0 / norm(a))
}

/// Produit mixte a·(b×c) : positif si (a, b, c) est CCW vu de l'extérieur.
pub fn det(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    dot(a, cross(b, c))
}

/// Angle solide d'un triangle sphérique unitaire (formule d'Van Oosterom-Strackee).
pub fn solid_angle(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    2.0 * det(a, b, c).abs().atan2(1.0 + dot(a, b) + dot(b, c) + dot(c, a))
}

/// Circoncentre sphérique : point équidistant des trois sommets, du côté du triangle.
pub fn circumcenter(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let n = cross(sub(b, a), sub(c, a));
    let sign = if dot(n, a) < 0.0 { -1.0 } else { 1.0 };
    normalize(scale(n, sign))
}
