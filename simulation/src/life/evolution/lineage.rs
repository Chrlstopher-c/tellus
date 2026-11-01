//! Table des lignées : un identifiant par scission, moments des traits, phylogénie enregistrée.

use super::genome::{Genome, NT};
use crate::life::fauna::Cohort;

pub type LineageId = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct Lineage {
    pub id: LineageId,
    pub parent: Option<LineageId>,
    pub born_tick: u64,
    pub split_tick: Option<u64>,
    pub extinct_tick: Option<u64>,
    pub mu: Genome,
    pub var: Genome,
    pub n_total: f64,
    pub isolation_yr: f64,
    pub low_yr: f64,
}

/// Nœud de phylogénie : une lignée fille issue d'un parent (ou fondatrice si `parent` est `None`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhyloNode {
    pub parent: Option<LineageId>,
    pub child: LineageId,
    pub tick: u64,
    pub cell: u32,
}

#[derive(Debug, Clone, Default)]
pub struct LineageTable {
    lineages: Vec<Lineage>,
    phylogeny: Vec<PhyloNode>,
}

impl LineageTable {
    /// Crée une lignée ; son identifiant est son rang d'apparition (jamais codé à la main).
    pub fn create(&mut self, traits: Genome, parent: Option<LineageId>, tick: u64, cell: u32) -> LineageId {
        let id = self.lineages.len() as LineageId;
        self.lineages.push(Lineage {
            id,
            parent,
            born_tick: tick,
            split_tick: parent.map(|_| tick),
            extinct_tick: None,
            mu: traits,
            var: [0.0; NT],
            n_total: 0.0,
            isolation_yr: 0.0,
            low_yr: 0.0,
        });
        self.phylogeny.push(PhyloNode {
            parent,
            child: id,
            tick,
            cell,
        });
        id
    }

    pub fn get(&self, id: LineageId) -> Option<&Lineage> {
        self.lineages.get(id as usize)
    }

    pub fn get_mut(&mut self, id: LineageId) -> Option<&mut Lineage> {
        self.lineages.get_mut(id as usize)
    }

    pub fn is_alive(&self, id: LineageId) -> bool {
        self.get(id).is_some_and(|l| l.extinct_tick.is_none())
    }

    pub fn alive_ids(&self) -> Vec<LineageId> {
        self.lineages
            .iter()
            .filter(|l| l.extinct_tick.is_none())
            .map(|l| l.id)
            .collect()
    }

    pub fn alive_count(&self) -> usize {
        self.lineages.iter().filter(|l| l.extinct_tick.is_none()).count()
    }

    pub fn count(&self) -> usize {
        self.lineages.len()
    }

    pub fn phylogeny(&self) -> &[PhyloNode] {
        &self.phylogeny
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lineage> {
        self.lineages.iter()
    }
}

/// Moments de chaque lignée (moyenne et variance pondérées par l'effectif), recalculés depuis les cohortes.
pub fn refresh_moments(table: &mut LineageTable, cohorts: &[Vec<Cohort>]) {
    let count = table.count();
    let mut sum_n = vec![0.0_f64; count];
    let mut sum_t = vec![[0.0_f64; NT]; count];
    let mut sum_t2 = vec![[0.0_f64; NT]; count];
    for cell in cohorts {
        for c in cell {
            let k = c.lineage as usize;
            sum_n[k] += c.n;
            for i in 0..NT {
                sum_t[k][i] += c.n * c.genome[i];
                sum_t2[k][i] += c.n * c.genome[i] * c.genome[i];
            }
        }
    }
    for (k, lineage) in table.lineages.iter_mut().enumerate() {
        lineage.n_total = sum_n[k];
        if sum_n[k] > 0.0 {
            for i in 0..NT {
                let mean = sum_t[k][i] / sum_n[k];
                lineage.mu[i] = mean;
                lineage.var[i] = (sum_t2[k][i] / sum_n[k] - mean * mean).max(0.0);
            }
        }
    }
}
