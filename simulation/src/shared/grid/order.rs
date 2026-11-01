//! Réordonnancement des cellules le long d'une courbe de Morton 3D (localité mémoire).

use super::geom::Vec3;

const COORD_MAX: f64 = ((1_u64 << 21) - 1) as f64;

/// Permutation : position k dans l'ordre de Morton → indice d'origine.
pub fn morton_order(positions: &[Vec3]) -> Vec<u32> {
    let mut keys: Vec<(u64, u32)> = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| (morton_key(p), i as u32))
        .collect();
    keys.sort_unstable();
    keys.into_iter().map(|(_, i)| i).collect()
}

/// Applique la permutation aux sommets et renumérote les triangles.
pub fn apply(order: &[u32], positions: Vec<Vec3>, triangles: Vec<[u32; 3]>) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut new_of_old = vec![0_u32; order.len()];
    for (new, &old) in order.iter().enumerate() {
        new_of_old[old as usize] = new as u32;
    }
    let reordered = order.iter().map(|&old| positions[old as usize]).collect();
    let remapped = triangles.iter().map(|t| t.map(|v| new_of_old[v as usize])).collect();
    (reordered, remapped)
}

fn morton_key(p: Vec3) -> u64 {
    let q = |x: f64| ((x * 0.5 + 0.5).clamp(0.0, 1.0) * COORD_MAX) as u64;
    spread(q(p[0])) | (spread(q(p[1])) << 1) | (spread(q(p[2])) << 2)
}

/// Écarte les 21 bits de poids faible pour les intercaler à pas de 3.
fn spread(value: u64) -> u64 {
    let mut x = value & 0x1f_ffff;
    x = (x | x << 32) & 0x1f_0000_0000_ffff;
    x = (x | x << 16) & 0x1f_0000_ff00_00ff;
    x = (x | x << 8) & 0x100f_00f0_0f00_f00f;
    x = (x | x << 4) & 0x10c3_0c30_c30c_30c3;
    (x | x << 2) & 0x1249_2492_4924_9249
}
