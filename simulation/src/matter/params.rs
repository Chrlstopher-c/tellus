//! Constantes du domaine matière (`docs/recherche/nutriments.md`). Stocks : GtC, GtN, GtP, Tmol O₂, km³ H₂O.
//! Concentrations de surface en g m⁻², flux annuels sauf mention. Les valeurs marquées « calage » sont des choix
//! de modèle réduit, documentés dans `ARCHITECTURE.md` ; elles ne sont pas des mesures.

/// Pression partielle de CO₂ préindustrielle de référence, ppm.
pub const PPM_REF: f64 = 280.0;
/// Mol d'O₂ libérées par GtC enfouie (1 GtC ≈ 83,3 Tmol ; photosynthèse CO₂ + H₂O → CH₂O + O₂ à 1 pour 1).
pub const TMOL_O2_PER_GTC: f64 = 83.3;
/// Stock d'O₂ actuel, Tmol (≈ 21 % en mole, nutriments § 0).
pub const O2_PAL_TMOL: f64 = 3.7e7;
/// Fraction molaire d'O₂ actuelle.
pub const O2_PAL_FRACTION: f64 = 0.21;
/// Réserve de carbone mantellique (source volcanique), GtC.
pub const MANTLE_CARBON_GTC: f64 = 1.0e9;
/// Réserve d'azote atmosphérique N₂ (quasi infinie à l'échelle de Tellus), GtN.
pub const N2_RESERVE_GTN: f64 = 3.9e6;
/// Réserve d'O₂ source (photolyse de l'eau), Tmol.
pub const O2_SOURCE_TMOL: f64 = 1.0e9;
/// Réserve d'apatite (phosphore altérable), GtP.
pub const APATITE_RESERVE_GTP: f64 = 1.0e4;
/// Vapeur atmosphérique initiale, km³ (table nutriments § 1.1).
pub const VAPOUR_KM3: f64 = 12_900.0;
/// Eau océanique totale, km³ (table nutriments § 1.1).
pub const OCEAN_WATER_KM3: f64 = 1.338e9;
/// Capacité de rétention du sol, mm d'eau.
pub const SOIL_CAP_MM: f64 = 150.0;
/// Évapotranspiration potentielle par kelvin au-dessus de −5 °C, mm an⁻¹ (calage, ordre de Hamon).
pub const PET_MM_PER_K_YR: f64 = 60.0;
/// Décalage de température de l'ETP, K.
pub const PET_OFFSET_C: f64 = 5.0;
/// Conversion matière sèche → carbone (Miami, nutriments § 2.2).
pub const MIAMI_GC_PER_GDM: f64 = 0.45;
/// Effet fertilisant du CO₂ : f = 1 + β ln(C/C₀), β ≈ 0,4 (nutriments § 2.2).
pub const CO2_FERT_BETA: f64 = 0.4;
/// Éclairement au-delà duquel la lumière ne limite plus la NPP, W m⁻² (calage).
pub const LIGHT_SAT_W_M2: f64 = 100.0;
/// Température de référence de la respiration du sol, °C.
pub const T_REF_C: f64 = 15.0;
/// Q10 de la décomposition (nutriments § 2.3).
pub const Q10_SOIL: f64 = 2.0;
/// Taux de renouvellement de la végétation, an⁻¹ (τ = 10 ans).
pub const K_VEG_PER_YR: f64 = 0.1;
/// Décomposition du pool rapide (litière + sol actif), an⁻¹ (RothC, réduit à deux pools).
pub const K_FAST_PER_YR: f64 = 0.3;
/// Décomposition du pool lent (humus), an⁻¹.
pub const K_SLOW_PER_YR: f64 = 0.02;
/// Fraction de la décomposition du pool rapide humifiée vers le pool lent (RothC, ≈ 46 %).
pub const HUMIFICATION: f64 = 0.46;
/// Rapport C:N de la végétation et de la litière (masse), calage (nutriments § 6).
pub const CN_VEG: f64 = 60.0;
/// Rapport C:P de la végétation (masse), calage.
pub const CP_VEG: f64 = 800.0;
/// Rapport C:N de la matière organique marine (Redfield 106:16, en masse).
pub const CN_OCEAN: f64 = 5.68;
/// Rapport C:P de la matière organique marine (Redfield 106:1, en masse).
pub const CP_OCEAN: f64 = 41.1;
/// Productivité marine potentielle maximale, gC m⁻² an⁻¹ (calage).
pub const OCEAN_NPP_MAX_GC_M2_YR: f64 = 300.0;
/// Demi-saturation de l'azote océanique, g m⁻² (calage).
pub const K_N_OCEAN_G_M2: f64 = 3.0;
/// Demi-saturation du phosphore océanique, g m⁻² (calage).
pub const K_P_OCEAN_G_M2: f64 = 0.3;
/// Azote dissous initial de la couche de mélange, g m⁻² (nutriments § 3 : nitrate ≈ 30 µmol kg⁻¹).
pub const INIT_DIN_G_M2: f64 = 30.0;
/// Phosphore dissous initial de la couche de mélange, g m⁻² (≈ 2 µmol kg⁻¹).
pub const INIT_DIP_G_M2: f64 = 4.4;
/// Fraction de la production marine exportée vers le fond (pompe biologique, nutriments § 2.5).
pub const EXPORT_RATIO: f64 = 0.15;
/// Temps de remontée profonde → surface pour le carbone, ans (calage : export / τ ≈ 7 500 GtC).
pub const TAU_UPWELL_C_YR: f64 = 1_000.0;
/// Temps de remontée pour l'azote, ans (≈ 520 GtN de nitrate profond).
pub const TAU_UPWELL_N_YR: f64 = 400.0;
/// Temps de remontée pour le phosphore, ans (≈ 72 GtP).
pub const TAU_UPWELL_P_YR: f64 = 400.0;
/// Temps d'équilibre de la couche de mélange avec l'atmosphère, ans (nutriments § 2.5 : ≈ 1 an).
pub const TAU_MIXED_LAYER_YR: f64 = 1.0;
/// Carbone inorganique de référence de la couche de mélange, gC m⁻² (≈ 2 mmol kg⁻¹ sur 70 m).
pub const DIC_REF_GC_M2: f64 = 1_722.0;
/// Sensibilité de la pCO₂ de surface à la température, K⁻¹ (solubilité, ≈ 4 % K⁻¹).
pub const SOLUBILITY_PER_K: f64 = 0.0423;
/// Facteur de Revelle : d ln pCO₂ / d ln DIC (nutriments § 2.5).
pub const REVELLE: f64 = 10.0;
/// Fixation biologique maximale par biome, gN m⁻² an⁻¹ (Terre : 0,7 à 2 g m⁻² an⁻¹, nutriments § 3 ; plafond de
/// calage). Elle s'active quand la NPP est limitée par l'azote. Son coût énergétique (≈ 12 gC par gN, non couplé à
/// la NPP au MVP) est une limite connue du modèle.
pub const FIX_CAP_GN_M2_YR: f64 = 3.0;
/// Stock profond d'azote et de phosphore d'un océan en régime (export / τ de remontée), GtN et GtP.
pub const INIT_DEEP_N_GTN: f64 = 600.0;
pub const INIT_DEEP_P_GTP: f64 = 85.0;
/// Dénitrification de l'azote minéral, an⁻¹ (calage).
pub const DENITRIFICATION_PER_YR: f64 = 0.02;
/// Lessivage de l'azote minéral pour un ruissellement unitaire, an⁻¹ (calage).
pub const LEACH_N_PER_YR: f64 = 0.1;
/// Lessivage du phosphore labile pour un ruissellement unitaire, an⁻¹ (calage).
pub const LEACH_P_PER_YR: f64 = 0.02;
/// Altération de l'apatite pour un facteur de climat égal à 1, an⁻¹ (≈ 0,015 GtP an⁻¹ à 1e4 GtP).
pub const APATITE_K_PER_YR: f64 = 1.5e-6;
/// Enfouissement du phosphore profond vers les sédiments, an⁻¹ (τ ≈ 5 000 ans, calage).
pub const P_BURIAL_K_PER_YR: f64 = 2.0e-4;
/// Enfouissement organique, fraction de la NPP mondiale (≈ 0,15 GtC an⁻¹ pour 100 GtC an⁻¹).
pub const BURIAL_FRACTION: f64 = 0.0015;
/// Fraction du carbone organique enfoui recyclé vers l'atmosphère (subduction), le reste est une source nette d'O₂.
pub const ORGANIC_RECYCLE: f64 = 0.95;
/// Altération des silicates à pCO₂ de référence et climat de référence, GtC an⁻¹ (calage : égal au dégazage net).
pub const SILICATE_REF_GTC_YR: f64 = 0.19;
/// Dégazage volcanique de référence, GtC an⁻¹ (nutriments § 2.6).
pub const VOLCANIC_GTC_YR: f64 = 0.2;
/// Exposant de la pCO₂ dans l'altération (Walker-Hays-Kasting, GEOCARB : 0,3 à 0,5).
pub const WEATHER_CO2_EXP: f64 = 0.4;
/// Sensibilité de l'altération à la température, K⁻¹ (Arrhenius, Ea ≈ 60-90 kJ mol⁻¹).
pub const WEATHER_TEMP_PER_K: f64 = 0.09;
/// Échelle de relief de l'altération, m.
pub const ELEVATION_REF_M: f64 = 2_000.0;
/// Pas géologique maximal, ans (super-tick 100-1 000 ans, nutriments § 8).
pub const GEO_MAX_SUBSTEP_YR: f64 = 100.0;
/// Nombre de sous-pas biologiques quand Δt ≤ 1 an.
pub const BIO_SUBSTEPS: usize = 12;

/// Paramètres de départ de la matière. Par défaut : planète jeune, CO₂ élevé, dégazage de référence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatterConfig {
    /// CO₂ atmosphérique initial, ppm (inventaire volatil de la planète jeune).
    pub co2_ppm: f64,
    /// Multiplicateur du dégazage volcanique (1 = référence ; 2 = doublement, REGLES § 8).
    pub volcanic_factor: f64,
}

impl Default for MatterConfig {
    fn default() -> Self {
        Self {
            co2_ppm: 2_000.0,
            volcanic_factor: 1.0,
        }
    }
}
