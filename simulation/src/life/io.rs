//! Contrat d'intégration du vivant (docs/REGLES.md § 12) : forçages empruntés et bilan scalaire.
//! Les champs par cellule en sortie se lisent sur `LifeState` (noms fixes du contrat).

use super::ledger::Flows;

/// Forçages empruntés aux autres domaines, un élément par cellule (ordre de la grille).
#[derive(Debug, Clone, Copy)]
pub struct LifeInputs<'a> {
    pub temperature_c: &'a [f32],
    pub precip_mm_yr: &'a [f32],
    pub soil_moisture: &'a [f32],
    pub ice_fraction: &'a [f32],
    pub is_ocean: &'a [bool],
    pub npp_cap_gc_m2_yr: &'a [f32],
    pub n_avail: &'a [f32],
    pub p_avail: &'a [f32],
    /// Azote disponible en océan (0-1 du contrat), limite le plancton.
    pub nutrient_ocean: &'a [f32],
    pub o2_fraction: f64,
    pub co2_ppm: f64,
}

impl LifeInputs<'_> {
    /// Vrai si chaque tranche a exactement `cells` éléments et si les scalaires sont finis.
    pub fn matches(&self, cells: usize) -> bool {
        let lengths = [
            self.temperature_c.len(),
            self.precip_mm_yr.len(),
            self.soil_moisture.len(),
            self.ice_fraction.len(),
            self.is_ocean.len(),
            self.npp_cap_gc_m2_yr.len(),
            self.n_avail.len(),
            self.p_avail.len(),
            self.nutrient_ocean.len(),
        ];
        lengths.iter().all(|&n| n == cells) && self.o2_fraction.is_finite() && self.co2_ppm.is_finite()
    }
}

/// Bilan scalaire d'un pas. `input_error` signale des forçages incohérents : le pas n'a alors pas eu lieu.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LifeReport {
    pub input_error: bool,
    pub photosynth_present: bool,
    pub lineages_alive: usize,
    pub mean_veg_cover: f64,
    pub flora_biomass_kgc: f64,
    pub fauna_biomass_kgc: f64,
    pub flows: Flows,
    pub speciations: u32,
    pub extinctions: u32,
    /// Écart relatif maximal entre les stocks du registre et l'état : doit rester ≤ 1e-9.
    pub conservation_error: f64,
}

impl LifeReport {
    pub fn rejected() -> Self {
        Self {
            input_error: true,
            ..Self::default()
        }
    }
}
