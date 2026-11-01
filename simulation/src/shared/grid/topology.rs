//! Anneaux de triangles CCW autour de chaque sommet (CSR), voisinage et adjacence des triangles.

/// Triangles incidents à chaque sommet, en ordre CCW, au format CSR.
pub struct Rings {
    pub offsets: Vec<u32>,
    pub triangles: Vec<u32>,
}

impl Rings {
    pub fn of(&self, vertex: usize) -> &[u32] {
        &self.triangles[self.offsets[vertex] as usize..self.offsets[vertex + 1] as usize]
    }
}

pub fn build_rings(vertex_count: usize, tris: &[[u32; 3]]) -> Rings {
    let mut valence = vec![0_u32; vertex_count];
    for tri in tris {
        for &v in tri {
            valence[v as usize] += 1;
        }
    }
    let offsets = prefix_sums(&valence);
    let mut incident = vec![0_u32; offsets[vertex_count] as usize];
    let mut fill = offsets.clone();
    for (t, tri) in tris.iter().enumerate() {
        for &v in tri {
            incident[fill[v as usize] as usize] = t as u32;
            fill[v as usize] += 1;
        }
    }
    let mut triangles = Vec::with_capacity(incident.len());
    for (v, range) in offsets.windows(2).enumerate() {
        let slice = &incident[range[0] as usize..range[1] as usize];
        triangles.extend(order_ring(v as u32, tris, slice));
    }
    Rings { offsets, triangles }
}

fn prefix_sums(counts: &[u32]) -> Vec<u32> {
    std::iter::once(0)
        .chain(counts.iter().scan(0_u32, |acc, &c| {
            *acc += c;
            Some(*acc)
        }))
        .collect()
}

/// Parcourt les triangles incidents à `v` en suivant les arêtes partagées, dans l'ordre CCW.
fn order_ring(v: u32, tris: &[[u32; 3]], incident: &[u32]) -> Vec<u32> {
    let start = incident[0];
    let mut ring = vec![start];
    let mut current = start;
    for _ in 1..incident.len() {
        let before = prev_vertex(tris[current as usize], v);
        current = incident
            .iter()
            .copied()
            .find(|&t| next_vertex(tris[t as usize], v) == before)
            .expect("anneau de triangles non fermé");
        ring.push(current);
    }
    let last = tris[current as usize];
    debug_assert_eq!(prev_vertex(last, v), next_vertex(tris[start as usize], v));
    ring
}

fn slot_of(tri: [u32; 3], v: u32) -> usize {
    tri.iter().position(|&x| x == v).expect("sommet absent du triangle")
}

fn next_vertex(tri: [u32; 3], v: u32) -> u32 {
    tri[(slot_of(tri, v) + 1) % 3]
}

fn prev_vertex(tri: [u32; 3], v: u32) -> u32 {
    tri[(slot_of(tri, v) + 2) % 3]
}

/// Voisins CSR : même décalage que `rings.offsets`, voisins en ordre CCW.
pub fn neighbors(rings: &Rings, tris: &[[u32; 3]]) -> Vec<u32> {
    let mut out = Vec::with_capacity(rings.triangles.len());
    for (v, range) in rings.offsets.windows(2).enumerate() {
        for &t in &rings.triangles[range[0] as usize..range[1] as usize] {
            out.push(next_vertex(tris[t as usize], v as u32));
        }
    }
    out
}

/// Pour chaque triangle, le voisin de l'arête opposée au sommet `i` (indice `i` du tableau).
pub fn triangle_adjacency(rings: &Rings, tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    tris.iter()
        .map(|&t| [0_usize, 1, 2].map(|i| across(rings, tris, t, i)))
        .collect()
}

fn across(rings: &Rings, tris: &[[u32; 3]], tri: [u32; 3], opposite: usize) -> u32 {
    let v = tri[(opposite + 2) % 3];
    let next = tri[(opposite + 1) % 3];
    rings
        .of(v as usize)
        .iter()
        .copied()
        .find(|&s| next_vertex(tris[s as usize], v) == next)
        .expect("triangle voisin introuvable")
}
