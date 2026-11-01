//! Vue du vivant pour l'inspecteur : nom du biome (Whittaker, calculé comme le domaine vivant), rôle et effectif des
//! lignées présentes dans une cellule. Lecture seule : aucune règle nouvelle ici.

use crate::life::evolution::idx;
use crate::life::fauna::Cohort;
use crate::life::flora::biome::whittaker_class;
use crate::life::naming::species_name;
use crate::life::LifeState;

/// Libellés des classes de Whittaker, dans l'ordre de `life::flora::biome` (0 Inlandsis … 9 Forêt tropicale humide).
const BIOME_NAMES: [&str; 10] = [
    "Inlandsis",
    "Toundra",
    "Taïga",
    "Steppe froide",
    "Prairie",
    "Forêt tempérée",
    "Désert chaud",
    "Savane",
    "Forêt saisonnière",
    "Forêt tropicale humide",
];
/// Lignées affichées au plus dans l'inspecteur (la première ligne est la plus peuplée).
const SPECIES_SHOWN: usize = 6;
/// Régime carné au-delà duquel une faune est prédatrice (trait `DIET` de 0 à 1).
const PREDATOR_DIET: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Producer,
    Herbivore,
    Predator,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Producer => "producer",
            Self::Herbivore => "herbivore",
            Self::Predator => "predator",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesLine {
    pub id: u32,
    pub name: String,
    pub role: Role,
    pub population: f64,
}

/// Nom du biome d'une cellule terrestre selon son état (température, précipitations, couvert).
pub fn biome_for(temp_c: f64, precip_mm: f64, cover: f64) -> &'static str {
    BIOME_NAMES[usize::from(whittaker_class(temp_c, precip_mm, cover))]
}

/// Nombre de lignées présentes dans la cellule, et les `SPECIES_SHOWN` plus peuplées.
pub fn species_in_cell(life: &LifeState, cell: usize) -> (usize, Vec<SpeciesLine>) {
    let mut lines: Vec<SpeciesLine> = Vec::new();
    for cohort in life.cohorts(cell) {
        match lines.iter_mut().find(|line| line.id == cohort.lineage) {
            Some(line) => line.population += cohort.n,
            None => lines.push(SpeciesLine {
                id: cohort.lineage,
                name: species_name(cohort.lineage),
                role: role_of(cohort),
                population: cohort.n,
            }),
        }
    }
    let count = lines.len();
    lines.sort_by(|a, b| b.population.total_cmp(&a.population).then(a.id.cmp(&b.id)));
    lines.truncate(SPECIES_SHOWN);
    (count, lines)
}

fn role_of(cohort: &Cohort) -> Role {
    if cohort.is_producer() {
        Role::Producer
    } else if cohort.genome[idx::DIET] >= PREDATOR_DIET {
        Role::Predator
    } else {
        Role::Herbivore
    }
}
