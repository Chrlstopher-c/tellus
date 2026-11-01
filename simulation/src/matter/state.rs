//! État du domaine matière : un registre de stocks par élément (`shared::ledger`), géométrie et masque océanique.
//! Seule primitive de changement : `Element::flow`, limitée au stock source (REGLES § 0.3).

use super::carbon;
use super::error::MatterError;
use super::inputs::MatterInputs;
use super::nitrogen;
use super::oxygen;
use super::params::MatterConfig;
use super::phosphorus;
use super::water;
use crate::shared::grid::Grid;
use crate::shared::ledger::Ledger;
use crate::shared::units::{GRAMS_PER_GT, GTC_PER_PPM};

/// Écart relatif au-delà duquel une demande de transfert est comptée comme manque (et non comme arrondi).
const SHORTFALL_TOLERANCE: f64 = 1e-9;

/// Disposition d'un élément : `kinds` blocs de `cells` stocks par cellule, puis `globals` réservoirs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Layout {
    pub(crate) cells: usize,
    pub(crate) kinds: usize,
    pub(crate) globals: usize,
}

impl Layout {
    pub(crate) fn at(self, kind: usize, cell: usize) -> usize {
        kind * self.cells + cell
    }

    pub(crate) fn glob(self, global: usize) -> usize {
        self.kinds * self.cells + global
    }

    pub(crate) fn len(self) -> usize {
        self.kinds * self.cells + self.globals
    }
}

/// Stocks d'un élément chimique et son registre de transferts.
pub(crate) struct Element {
    pub(crate) layout: Layout,
    pub(crate) stocks: Vec<f64>,
    ledger: Ledger,
    pub(crate) shortfalls: u64,
    pub(crate) anomalies: u64,
}

impl Element {
    pub(crate) fn from_stocks(layout: Layout, stocks: Vec<f64>) -> Self {
        let ledger = Ledger::open(&stocks);
        Self {
            layout,
            stocks,
            ledger,
            shortfalls: 0,
            anomalies: 0,
        }
    }

    pub(crate) fn get(&self, idx: usize) -> f64 {
        self.stocks[idx]
    }

    pub(crate) fn total(&self) -> f64 {
        self.stocks.iter().sum()
    }

    /// Déplace `requested` de `from` vers `to`, limité au stock source. Un manque est compté ; une valeur non finie
    /// ou négative est refusée et comptée comme anomalie.
    pub(crate) fn flow(&mut self, from: usize, to: usize, requested: f64) -> f64 {
        if !requested.is_finite() || requested < 0.0 {
            self.anomalies += 1;
            return 0.0;
        }
        let available = self.stocks[from].max(0.0);
        if requested - available > SHORTFALL_TOLERANCE * requested {
            self.shortfalls += 1;
        }
        let amount = requested.min(available);
        self.ledger.transfer(&mut self.stocks, from, to, amount)
    }

    /// Transfert signé : `signed ≥ 0` va de `a` vers `b`, sinon de `b` vers `a`.
    pub(crate) fn exchange(&mut self, a: usize, b: usize, signed: f64) {
        if signed >= 0.0 {
            self.flow(a, b, signed);
        } else {
            self.flow(b, a, -signed);
        }
    }

    /// Répartit `amount` depuis `from` vers le bloc `kind` des cellules océaniques, au prorata de l'aire.
    pub(crate) fn spread_over_ocean(&mut self, geo: &Geometry, from: usize, kind: usize, amount: f64) {
        for i in geo.ocean_cells() {
            let target = self.layout.at(kind, i);
            self.flow(from, target, amount * geo.areas_m2[i] / geo.ocean_area_m2);
        }
    }

    /// Répartit `amount` depuis `from` vers le bloc `kind` des cellules terrestres, au prorata de l'aire.
    pub(crate) fn spread_over_land(&mut self, geo: &Geometry, from: usize, kind: usize, amount: f64) {
        for i in geo.land_cells() {
            let target = self.layout.at(kind, i);
            self.flow(from, target, amount * geo.areas_m2[i] / geo.land_area_m2);
        }
    }

    /// Rouvre le registre sur l'état courant (tests : initialisation directe des stocks).
    #[cfg(test)]
    pub(crate) fn reopen(&mut self) {
        self.ledger = Ledger::open(&self.stocks);
    }
}

/// Géométrie fixe : aires des cellules, masque océan-continent, aires totales.
pub(crate) struct Geometry {
    pub(crate) areas_m2: Vec<f64>,
    pub(crate) is_ocean: Vec<bool>,
    pub(crate) land_area_m2: f64,
    pub(crate) ocean_area_m2: f64,
}

impl Geometry {
    fn new(areas_m2: &[f64], is_ocean: &[bool]) -> Self {
        let (mut land, mut ocean) = (0.0, 0.0);
        for (&area, &sea) in areas_m2.iter().zip(is_ocean) {
            if sea {
                ocean += area;
            } else {
                land += area;
            }
        }
        Self {
            areas_m2: areas_m2.to_vec(),
            is_ocean: is_ocean.to_vec(),
            land_area_m2: land,
            ocean_area_m2: ocean,
        }
    }

    pub(crate) fn cell_count(&self) -> usize {
        self.areas_m2.len()
    }

    pub(crate) fn land_cells(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.is_ocean.len()).filter(move |&i| !self.is_ocean[i])
    }

    pub(crate) fn ocean_cells(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.is_ocean.len()).filter(move |&i| self.is_ocean[i])
    }
}

/// État complet de la matière d'une planète. Les quatre tranches par cellule sont les sorties du domaine (§ 12),
/// recalculées en début de chaque tick.
pub struct MatterState {
    pub(crate) geometry: Geometry,
    pub(crate) carbon: Element,
    pub(crate) nitrogen: Element,
    pub(crate) phosphorus: Element,
    pub(crate) oxygen: Element,
    pub(crate) water: Element,
    pub(crate) volcanic_factor: f64,
    pub npp_cap_gc_m2_yr: Vec<f32>,
    pub n_avail: Vec<f32>,
    pub p_avail: Vec<f32>,
    pub nutrient_ocean: Vec<f32>,
}

impl MatterState {
    pub fn cell_count(&self) -> usize {
        self.geometry.cell_count()
    }

    /// pCO₂ atmosphérique, ppm.
    pub fn co2_ppm(&self) -> f64 {
        self.carbon.get(self.carbon.layout.glob(carbon::ATM)) / GTC_PER_PPM
    }

    /// Stock d'O₂ atmosphérique, Tmol.
    pub fn o2_tmol(&self) -> f64 {
        self.oxygen.get(self.oxygen.layout.glob(oxygen::ATM))
    }

    pub(crate) fn elements(&self) -> [&Element; 5] {
        [
            &self.carbon,
            &self.nitrogen,
            &self.phosphorus,
            &self.oxygen,
            &self.water,
        ]
    }

    pub(crate) fn totals(&self) -> [f64; 5] {
        self.elements().map(Element::total)
    }
}

fn validate_config(config: &MatterConfig) -> Result<(), MatterError> {
    if !config.co2_ppm.is_finite() || config.co2_ppm < 0.0 {
        return Err(MatterError::InvalidConfig("co2_ppm doit être fini et positif"));
    }
    if !config.volcanic_factor.is_finite() || config.volcanic_factor < 0.0 {
        return Err(MatterError::InvalidConfig("volcanic_factor doit être fini et positif"));
    }
    Ok(())
}

/// Planète jeune : CO₂ d'inventaire, océan formé et sans vie, sols et atmosphère d'azote et de phosphore au départ.
pub fn init(config: &MatterConfig, grid: &Grid, inputs: &MatterInputs<'_>) -> Result<MatterState, MatterError> {
    validate_config(config)?;
    let areas_m2 = grid.areas_m2();
    let cells = areas_m2.len();
    inputs.validate(cells)?;
    if let Some(index) = areas_m2.iter().position(|a| !a.is_finite() || *a <= 0.0) {
        return Err(MatterError::NonFinite {
            field: "areas_m2",
            index,
        });
    }
    let geometry = Geometry::new(areas_m2, inputs.is_ocean);
    Ok(MatterState {
        carbon: carbon::initial(&geometry, inputs.temperature_c, config.co2_ppm),
        nitrogen: nitrogen::initial(&geometry),
        phosphorus: phosphorus::initial(&geometry),
        oxygen: oxygen::initial(cells),
        water: water::initial(&geometry),
        volcanic_factor: config.volcanic_factor,
        npp_cap_gc_m2_yr: vec![0.0; cells],
        n_avail: vec![0.0; cells],
        p_avail: vec![0.0; cells],
        nutrient_ocean: vec![0.0; cells],
        geometry,
    })
}

/// Stock en Gt réparti sur une surface → grammes par mètre carré.
pub(crate) fn per_area_g_m2(stock_gt: f64, area_m2: f64) -> f64 {
    stock_gt * GRAMS_PER_GT / area_m2
}

/// Grammes par mètre carré sur une surface → Gt.
pub(crate) fn gt_from_g_m2(value_g_m2: f64, area_m2: f64) -> f64 {
    value_g_m2 * area_m2 / GRAMS_PER_GT
}
