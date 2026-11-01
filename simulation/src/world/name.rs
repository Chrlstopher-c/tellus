//! Nom déterministe d'un monde généré, tiré de sa graine. Même algorithme que `nameFromSeed` côté interface
//! (`interface/src/world-setup/world-config.ts`) : les deux doivent concorder, voir les tests croisés.

const SYLLABLES: [&str; 16] = [
    "as", "ca", "lon", "ve", "yra", "or", "me", "thal", "is", "ar", "del", "un", "ka", "sel", "mi", "ron",
];
const MULTIPLIER: u32 = 2_246_822_519;
const INCREMENT: u32 = 3_266_489_917;

/// Flux à 32 bits de la fabrique de noms (mêmes constantes que l'interface).
struct NameStream {
    state: u32,
}

impl NameStream {
    fn next_u32(&mut self) -> u32 {
        self.state = (self.state ^ (self.state >> 15))
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(INCREMENT);
        self.state
    }

    fn next_index(&mut self, len: usize) -> usize {
        self.next_u32() as usize % len
    }
}

/// Deux ou trois syllabes, première lettre en capitale. La graine est réduite modulo 2³² comme `seed >>> 0`.
pub fn name_from_seed(seed: u64) -> String {
    let mut stream = NameStream { state: seed as u32 };
    let count = 2 + stream.next_index(2);
    let mut word = String::new();
    for _ in 0..count {
        word.push_str(SYLLABLES[stream.next_index(SYLLABLES.len())]);
    }
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => word,
    }
}
