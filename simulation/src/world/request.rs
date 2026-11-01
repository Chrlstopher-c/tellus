//! Requête de création d'un monde (`WorldConfig` de l'interface + niveau de grille), lue depuis du JSON.

use super::error::WorldError;
use serde_json::Value;

/// Niveau de grille par défaut (REGLES §13).
pub const DEFAULT_LEVEL: u32 = 7;
/// Plus grand niveau de grille supporté (L7 = 163 842 cellules).
pub const MAX_LEVEL: u32 = 7;
/// Plus grand entier exact représentable en nombre JavaScript.
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedSpec {
    pub seed: u64,
    pub water_percent: f64,
    pub size_percent: f64,
    pub distance_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Generated(GeneratedSpec),
    /// Nom affiché du corps (« Terre », « Vénus »…) ou identifiant (`earth`, `venus`…).
    Body(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldRequest {
    pub source: Source,
    pub level: u32,
}

impl WorldRequest {
    /// Lit `{source, level?}`. Un niveau absent vaut `DEFAULT_LEVEL` ; les bornes du niveau sont vérifiées à la création.
    pub fn from_json(text: &str) -> Result<Self, WorldError> {
        let value: Value = serde_json::from_str(text).map_err(|e| invalid(format!("JSON illisible ({e})")))?;
        let level = match value.get("level") {
            None | Some(Value::Null) => DEFAULT_LEVEL,
            Some(raw) => raw
                .as_u64()
                .and_then(|level| u32::try_from(level).ok())
                .ok_or_else(|| invalid("niveau entier attendu"))?,
        };
        let source = parse_source(value.get("source").ok_or_else(|| invalid("source absente"))?)?;
        Ok(Self { source, level })
    }
}

fn invalid(reason: impl Into<String>) -> WorldError {
    WorldError::InvalidRequest(reason.into())
}

fn parse_source(source: &Value) -> Result<Source, WorldError> {
    match source.get("kind").and_then(Value::as_str) {
        Some("generated") => Ok(Source::Generated(GeneratedSpec {
            seed: whole(source, "seed")?,
            water_percent: real(source, "waterPercent")?,
            size_percent: real(source, "sizePercent")?,
            distance_percent: real(source, "distancePercent")?,
        })),
        Some("body") => source
            .get("name")
            .and_then(Value::as_str)
            .map(|name| Source::Body(name.to_owned()))
            .ok_or_else(|| invalid("nom de corps absent")),
        _ => Err(invalid("source.kind inconnu (attendu : generated ou body)")),
    }
}

fn real(object: &Value, key: &str) -> Result<f64, WorldError> {
    object
        .get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
        .ok_or_else(|| invalid(format!("« {key} » absent ou non fini")))
}

fn whole(object: &Value, key: &str) -> Result<u64, WorldError> {
    let value = real(object, key)?;
    if value.fract() != 0.0 || value < 0.0 || value > MAX_SAFE_INTEGER {
        return Err(invalid(format!("« {key} » doit être un entier positif")));
    }
    Ok(value as u64)
}
