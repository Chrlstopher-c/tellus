//! Classification de Köppen à la demande (climat.md §9, « groupe majeur »). Entrées : moyennes mensuelles de
//! température (°C) et de précipitations (mm), 12 valeurs. Ordre des règles : E, puis B, puis A, puis C ou D.
//! Hors périmètre de cette étape : V-26 (accord ≥ 55 % avec la carte observée) et V-27 (BWh 10–20 % des terres).
//! Elles exigent une carte de référence et le vrai relief, absents du dépôt (MNT réduit et Köppen observé : E2.S2).
//! Un monde synthétique à continents en bandes ne peut ni confirmer ni infirmer ces chiffres.

/// Nombre de mois d'une année.
pub const MONTHS: usize = 12;

/// Groupe majeur de Köppen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KoppenGroup {
    /// Tropical humide (A).
    Tropical,
    /// Sec (B).
    Arid,
    /// Tempéré (C).
    Temperate,
    /// Continental (D).
    Continental,
    /// Polaire (E).
    Polar,
}

/// Classe un climat à partir de 12 moyennes mensuelles de température (°C) et de précipitations (mm).
pub fn classify(temp_c: &[f64; MONTHS], precip_mm: &[f64; MONTHS]) -> KoppenGroup {
    let coldest = temp_c.iter().copied().fold(f64::INFINITY, f64::min);
    let warmest = temp_c.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let annual_temp = temp_c.iter().sum::<f64>() / MONTHS as f64;
    let annual_precip: f64 = precip_mm.iter().sum();
    if warmest < 10.0 {
        KoppenGroup::Polar
    } else if annual_precip < arid_threshold_mm(annual_temp, temp_c, precip_mm) {
        KoppenGroup::Arid
    } else if coldest >= 18.0 {
        KoppenGroup::Tropical
    } else if coldest > -3.0 {
        KoppenGroup::Temperate
    } else {
        KoppenGroup::Continental
    }
}

/// Seuil de sécheresse B, mm/an : `20·T̄ + 280` si ≥ 70 % des pluies en demi-année chaude, `+ 0` si en froide,
/// `+ 140` sinon. Le demi-année « chaud » est la fenêtre de six mois la plus chaude.
fn arid_threshold_mm(annual_temp: f64, temp_c: &[f64; MONTHS], precip_mm: &[f64; MONTHS]) -> f64 {
    let warm_start = warmest_half_start(temp_c);
    let warm: f64 = (0..6).map(|k| precip_mm[(warm_start + k) % MONTHS]).sum();
    let total: f64 = precip_mm.iter().sum();
    let base = 20.0 * annual_temp;
    if total <= 0.0 {
        base
    } else if warm >= 0.7 * total {
        base + 280.0
    } else if warm <= 0.3 * total {
        base
    } else {
        base + 140.0
    }
}

/// Premier mois de la fenêtre de six mois consécutifs la plus chaude (moyenne).
fn warmest_half_start(temp_c: &[f64; MONTHS]) -> usize {
    let mean_of = |start: usize| (0..6).map(|k| temp_c[(start + k) % MONTHS]).sum::<f64>();
    (0..MONTHS).fold((0, f64::NEG_INFINITY), |best, start| {
        let value = mean_of(start);
        if value > best.1 {
            (start, value)
        } else {
            best
        }
    })
    .0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(value: f64) -> [f64; MONTHS] {
        [value; MONTHS]
    }

    /// Saison sinusoïdale : `mean ± amp`, maximum au mois 6 (juillet dans l'hémisphère nord).
    fn seasonal(mean: f64, amp: f64) -> [f64; MONTHS] {
        let mut out = [0.0; MONTHS];
        for (m, slot) in out.iter_mut().enumerate() {
            *slot = mean + amp * ((m as f64 - 6.0) / 12.0 * 2.0 * std::f64::consts::PI).cos();
        }
        out
    }

    #[test]
    fn equatorial_rainforest_is_tropical() {
        let t = flat(26.0);
        let p = flat(200.0);
        assert_eq!(classify(&t, &p), KoppenGroup::Tropical);
    }

    #[test]
    fn hot_desert_is_arid() {
        let t = flat(25.0);
        let p = flat(8.0);
        assert_eq!(classify(&t, &p), KoppenGroup::Arid);
    }

    #[test]
    fn mild_wet_climate_is_temperate() {
        let t = seasonal(12.0, 9.0);
        let p = flat(70.0);
        assert_eq!(classify(&t, &p), KoppenGroup::Temperate);
    }

    #[test]
    fn cold_winter_climate_is_continental() {
        let t = seasonal(5.0, 20.0);
        let p = flat(50.0);
        assert_eq!(classify(&t, &p), KoppenGroup::Continental);
    }

    #[test]
    fn cold_everywhere_is_polar() {
        let t = seasonal(-8.0, 6.0);
        let p = flat(20.0);
        assert_eq!(classify(&t, &p), KoppenGroup::Polar);
    }

    #[test]
    fn summer_rain_raises_the_dryness_threshold() {
        let t = seasonal(20.0, 4.0);
        let mut summer_rain = [5.0; MONTHS];
        for (m, slot) in summer_rain.iter_mut().enumerate().skip(3).take(6) {
            *slot = if m == 5 { 150.0 } else { 120.0 };
        }
        let annual: f64 = summer_rain.iter().sum();
        assert!(annual > 20.0 * 20.0 + 280.0 - 1.0, "fixture trop sèche : {annual}");
        assert_eq!(classify(&t, &summer_rain), KoppenGroup::Temperate);
    }
}
