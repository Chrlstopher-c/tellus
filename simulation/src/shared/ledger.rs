//! Transferts conservatifs entre stocks : jamais de stock négatif, clamps comptés.

/// Déplace `q` (≥ 0) de `src` vers `dst`, limité au stock disponible ; renvoie la quantité déplacée.
pub fn transfer(src: &mut f64, dst: &mut f64, q: f64) -> f64 {
    debug_assert!(q >= 0.0, "transfert négatif ou NaN : {q}");
    let moved = if q > 0.0 { q.min(src.max(0.0)) } else { 0.0 };
    *src -= moved;
    *dst += moved;
    moved
}

/// Registre cumulant entrées et sorties par élément, pour vérifier la conservation.
#[derive(Debug, Clone)]
pub struct Ledger {
    opening: Vec<f64>,
    inflow: Vec<f64>,
    outflow: Vec<f64>,
    clamps: u64,
}

impl Ledger {
    /// Ouvre le registre sur l'état initial des stocks.
    pub fn open(stocks: &[f64]) -> Self {
        Self {
            opening: stocks.to_vec(),
            inflow: vec![0.0; stocks.len()],
            outflow: vec![0.0; stocks.len()],
            clamps: 0,
        }
    }

    /// Transfert entre deux indices ; compte les clamps (quantité demandée non entièrement déplacée).
    pub fn transfer(&mut self, stocks: &mut [f64], from: usize, to: usize, q: f64) -> f64 {
        if from == to {
            return 0.0;
        }
        let mut src = stocks[from];
        let mut dst = stocks[to];
        let moved = transfer(&mut src, &mut dst, q);
        stocks[from] = src;
        stocks[to] = dst;
        self.outflow[from] += moved;
        self.inflow[to] += moved;
        if q > 0.0 && moved < q {
            self.clamps += 1;
        }
        moved
    }

    /// Nombre de transferts limités au stock source.
    pub fn clamps(&self) -> u64 {
        self.clamps
    }

    /// Écart relatif maximal entre stock réel et stock attendu (ouverture + entrées − sorties).
    pub fn imbalance(&self, stocks: &[f64]) -> f64 {
        (0..stocks.len())
            .map(|i| {
                let expected = self.opening[i] + self.inflow[i] - self.outflow[i];
                let scale = self.opening[i].abs() + self.inflow[i] + self.outflow[i];
                (stocks[i] - expected).abs() / scale.max(f64::MIN_POSITIVE)
            })
            .fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::unit_f64;

    #[test]
    fn ledger_conserves_exactly() {
        let mut stocks = [8.0, 0.0, 4.0];
        let mut ledger = Ledger::open(&stocks);
        ledger.transfer(&mut stocks, 0, 1, 2.5);
        ledger.transfer(&mut stocks, 2, 0, 1.0);
        ledger.transfer(&mut stocks, 1, 2, 0.25);
        assert_eq!(stocks.iter().sum::<f64>(), 12.0);
        assert_eq!(ledger.imbalance(&stocks), 0.0);
    }

    #[test]
    fn ledger_never_negative() {
        let mut stocks: Vec<f64> = (0..8).map(|i| unit_f64(5, 0, i, 0) * 10.0).collect();
        let total: f64 = stocks.iter().sum();
        let mut ledger = Ledger::open(&stocks);
        for step in 0..100_000_u32 {
            let from = (unit_f64(5, 1, step, 0) * 8.0) as usize % 8;
            let to = (unit_f64(5, 1, step, 1) * 8.0) as usize % 8;
            let q = unit_f64(5, 1, step, 2) * 20.0;
            ledger.transfer(&mut stocks, from, to, q);
            assert!(stocks.iter().all(|&s| s >= 0.0));
        }
        assert!((stocks.iter().sum::<f64>() - total).abs() / total < 1e-12);
        assert!(ledger.imbalance(&stocks) < 1e-12);
    }

    #[test]
    fn ledger_counts_clamps() {
        let mut stocks = [3.0, 0.0];
        let mut ledger = Ledger::open(&stocks);
        let moved = ledger.transfer(&mut stocks, 0, 1, 5.0);
        assert_eq!(moved, 3.0);
        assert_eq!(stocks, [0.0, 3.0]);
        assert_eq!(ledger.clamps(), 1);
        ledger.transfer(&mut stocks, 1, 0, 1.0);
        assert_eq!(ledger.clamps(), 1);
    }
}
