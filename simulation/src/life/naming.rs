//! Noms d'espèces pseudo-latins, déterministes depuis l'identifiant de lignée (style de mockup/ui.js).

use crate::shared::rng::hash;

const NAME_SEED: u64 = 0x7E11_0500_0000_0001;
const SYLLABLES: [&str; 16] = [
    "vel", "ora", "thal", "myr", "cae", "lun", "dor", "sev", "ith", "qua", "ner", "ulm", "ast", "phy", "ron", "elis",
];
const ENDINGS: [&str; 8] = ["ia", "us", "ax", "ella", "odon", "is", "ora", "ium"];
const EPITHETS: [&str; 10] = [
    "grandis",
    "australis",
    "velox",
    "umbra",
    "silvestris",
    "minor",
    "aquatica",
    "borealis",
    "ferox",
    "lenta",
];

/// Nom binomial déterministe : genre pseudo-latin (2 syllabes + désinence) et épithète.
pub fn species_name(id: u32) -> String {
    let draw = |stream: u32, len: usize| (hash(NAME_SEED, 0, id, stream) % len as u64) as usize;
    let genus = format!(
        "{}{}{}",
        SYLLABLES[draw(0, SYLLABLES.len())],
        SYLLABLES[draw(1, SYLLABLES.len())],
        ENDINGS[draw(2, ENDINGS.len())]
    );
    let epithet = EPITHETS[draw(3, EPITHETS.len())];
    let mut chars = genus.chars();
    match chars.next() {
        Some(head) => format!("{}{} {epithet}", head.to_uppercase(), chars.as_str()),
        None => epithet.to_string(),
    }
}
