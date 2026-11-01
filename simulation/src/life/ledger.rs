//! Registre de conservation du vivant : C, N, P répartis entre vivant, litière et échange avec la matière.

use super::flora::tilman::{CN_PLANT, CP_PLANT};

/// Flux d'un pas, en kgC (ou équivalent N, P pour le registre). Les flux internes (ingestion, dispersion,
/// fusion, reproduction) n'y figurent pas : ils ne changent aucun stock du vivant.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Flows {
    /// Carbone fixé depuis l'atmosphère (NPP).
    pub fixed: f64,
    /// Respiration animale et végétale rendue à l'atmosphère.
    pub resp: f64,
    /// Carbone brûlé par le feu, rendu à l'atmosphère.
    pub combusted: f64,
    /// Entrées de litière (turnover, égestion, mortalité, feu).
    pub litter_in: f64,
    /// Décomposition de la litière, rendue à l'atmosphère.
    pub decomposed: f64,
    pub intake_plant: f64,
    pub intake_animal: f64,
    pub assim: f64,
    pub egest: f64,
    pub dead: f64,
    pub fusions: u32,
    pub fire_cells: u32,
    /// Respirations limitées au stock disponible (compteur de clamp, doit rester marginal).
    pub clamps: u32,
}

impl Flows {
    pub fn add(&mut self, other: &Flows) {
        self.fixed += other.fixed;
        self.resp += other.resp;
        self.combusted += other.combusted;
        self.litter_in += other.litter_in;
        self.decomposed += other.decomposed;
        self.intake_plant += other.intake_plant;
        self.intake_animal += other.intake_animal;
        self.assim += other.assim;
        self.egest += other.egest;
        self.dead += other.dead;
        self.fusions += other.fusions;
        self.fire_cells += other.fire_cells;
        self.clamps += other.clamps;
    }
}

/// Stocks d'un élément : vivant, litière, quantité échangée avec la matière, quantité importée (lignées fondatrices).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Pools {
    pub living: f64,
    pub litter: f64,
    pub exported: f64,
    pub imported: f64,
}

impl Pools {
    /// Invariant : vivant + litière + exporté − importé reste nul, quel que soit le pas.
    pub fn total(&self) -> f64 {
        self.living + self.litter + self.exported - self.imported
    }

    /// Grandeur des stocks et flux, pour une tolérance relative (les écarts d'arrondi suivent les stocks).
    pub fn magnitude(&self) -> f64 {
        self.living.abs() + self.litter.abs() + self.exported.abs() + self.imported
    }

    fn apply(&mut self, fixed: f64, loss_external: f64, litter_in: f64, decomposed: f64) {
        self.living += fixed - loss_external - litter_in;
        self.litter += litter_in - decomposed;
        self.exported += loss_external + decomposed - fixed;
    }

    fn import(&mut self, amount: f64) {
        self.living += amount;
        self.imported += amount;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LifeLedger {
    pub carbon: Pools,
    pub nitrogen: Pools,
    pub phosphorus: Pools,
}

impl LifeLedger {
    /// Applique les flux d'un pas. L'azote et le phosphore suivent le carbone par leur stœchiométrie ;
    /// la respiration ne rend ni N ni P.
    pub fn apply_flows(&mut self, f: &Flows) {
        let loss = f.resp + f.combusted;
        self.carbon.apply(f.fixed, loss, f.litter_in, f.decomposed);
        self.nitrogen.apply(
            f.fixed / CN_PLANT,
            f.combusted / CN_PLANT,
            f.litter_in / CN_PLANT,
            f.decomposed / CN_PLANT,
        );
        self.phosphorus.apply(
            f.fixed / CP_PLANT,
            f.combusted / CP_PLANT,
            f.litter_in / CP_PLANT,
            f.decomposed / CP_PLANT,
        );
    }

    /// Une lignée fondatrice entre dans le vivant avec sa biomasse : c'est un apport, pas un flux interne.
    pub fn import_carbon(&mut self, biomass_kgc: f64) {
        self.carbon.import(biomass_kgc);
        self.nitrogen.import(biomass_kgc / CN_PLANT);
        self.phosphorus.import(biomass_kgc / CP_PLANT);
    }
}
