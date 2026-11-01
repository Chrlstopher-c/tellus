//! Lotka-Volterra isolé : intégrateur RK2 (point médian) et invariant V (docs/recherche/vivant.md § 3.2).
//! Sert au test d'isolement de la dynamique proie-prédateur ; le pas de la simulation ne l'emploie pas.

/// Paramètres : dx/dt = a·x − b·x·y ; dy/dt = c·x·y − d·y (x proies, y prédateurs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LotkaVolterra {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl LotkaVolterra {
    /// Second membre au point (x, y).
    pub fn rhs(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        [self.a * x - self.b * x * y, self.c * x * y - self.d * y]
    }

    /// Invariant conservé par le flot continu : V = c·x − d·ln x + b·y − a·ln y.
    pub fn invariant(&self, [x, y]: [f64; 2]) -> f64 {
        self.c * x - self.d * x.ln() + self.b * y - self.a * y.ln()
    }

    /// Pas RK2 (point médian) de durée `dt`.
    pub fn rk2_step(&self, state: [f64; 2], dt: f64) -> [f64; 2] {
        let k1 = self.rhs(state);
        let mid = [state[0] + 0.5 * dt * k1[0], state[1] + 0.5 * dt * k1[1]];
        let k2 = self.rhs(mid);
        [state[0] + dt * k2[0], state[1] + dt * k2[1]]
    }
}
