//! Super-tick géologique (nutriments §§ 2.6, 2.7, 5 ; REGLES § 6) : dégazage volcanique, altération des silicates
//! à la Walker-Hays-Kasting (dépend de pCO₂ et du climat), enfouissement organique et recyclage. Sous-pas ≤ 100 ans.

use super::carbon;
use super::forcing::Forcing;
use super::oxygen;
use super::params::{
    BURIAL_FRACTION, GEO_MAX_SUBSTEP_YR, ORGANIC_RECYCLE, PPM_REF, SILICATE_REF_GTC_YR, VOLCANIC_GTC_YR,
    WEATHER_CO2_EXP,
};
use super::state::Element;
use crate::shared::units::GTC_PER_PPM;

/// Flux géologiques de référence pour un tick : altération relative, enfouissement (GtC an⁻¹), volcanisme (GtC an⁻¹).
struct GeoRates {
    weathering: f64,
    burial_gtc_yr: f64,
    volcanic_gtc_yr: f64,
}

pub(crate) fn super_tick(c: &mut Element, o2: &mut Element, forcing: &Forcing, dt: f64, volcanic_factor: f64) {
    let steps = (dt / GEO_MAX_SUBSTEP_YR).ceil().max(1.0) as usize;
    let dt_sub = dt / steps as f64;
    let rates = GeoRates {
        weathering: forcing.weathering,
        burial_gtc_yr: BURIAL_FRACTION * forcing.npp_global_gtc(),
        volcanic_gtc_yr: VOLCANIC_GTC_YR * volcanic_factor,
    };
    for _ in 0..steps {
        geo_substep(c, o2, &rates, dt_sub);
    }
}

fn geo_substep(c: &mut Element, o2: &mut Element, rates: &GeoRates, dt: f64) {
    let (atm, mantle) = (c.layout.glob(carbon::ATM), c.layout.glob(carbon::MANTLE));
    let (sed_carb, sed_org) = (c.layout.glob(carbon::SED_CARB), c.layout.glob(carbon::SED_ORG));
    let ppm = c.get(atm) / GTC_PER_PPM;
    let silicate = SILICATE_REF_GTC_YR * (ppm / PPM_REF).powf(WEATHER_CO2_EXP) * rates.weathering;
    c.flow(atm, sed_carb, silicate * dt);
    c.flow(mantle, atm, rates.volcanic_gtc_yr * dt);
    let burial = rates.burial_gtc_yr * dt;
    c.flow(atm, sed_org, burial);
    oxygen::release(o2, burial);
    let recycled = ORGANIC_RECYCLE * burial;
    c.flow(sed_org, atm, recycled);
    oxygen::consume(o2, recycled);
}
