//! Oxygène atmosphérique (nutriments § 5). La photosynthèse et la respiration s'annulent à l'annuel : seul
//! l'enfouissement de carbone organique libère de l'O₂ net (1 mol par mol de C) ; son recyclage le consomme.

use super::params::{O2_PAL_FRACTION, O2_PAL_TMOL, O2_SOURCE_TMOL, TMOL_O2_PER_GTC};
use super::state::{Element, Layout};

pub(crate) const ATM: usize = 0;
pub(crate) const SOURCE: usize = 1;
pub(crate) const SINK: usize = 2;
const GLOBALS: usize = 3;

pub(crate) fn initial(cells: usize) -> Element {
    let layout = Layout {
        cells,
        kinds: 0,
        globals: GLOBALS,
    };
    let mut stocks = vec![0.0; layout.len()];
    stocks[layout.glob(SOURCE)] = O2_SOURCE_TMOL;
    Element::from_stocks(layout, stocks)
}

/// Libération d'O₂ par enfouissement de `carbon_gtc` GtC de carbone organique.
pub(crate) fn release(o2: &mut Element, carbon_gtc: f64) {
    let (source, atm) = (o2.layout.glob(SOURCE), o2.layout.glob(ATM));
    o2.flow(source, atm, carbon_gtc * TMOL_O2_PER_GTC);
}

/// Consommation d'O₂ par oxydation de `carbon_gtc` GtC de carbone organique recyclé.
pub(crate) fn consume(o2: &mut Element, carbon_gtc: f64) {
    let (atm, sink) = (o2.layout.glob(ATM), o2.layout.glob(SINK));
    o2.flow(atm, sink, carbon_gtc * TMOL_O2_PER_GTC);
}

/// Fraction molaire d'O₂ à partir du stock (21 % pour le stock actuel de 3,7e7 Tmol).
pub(crate) fn fraction(tmol: f64) -> f64 {
    O2_PAL_FRACTION * tmol / O2_PAL_TMOL
}
