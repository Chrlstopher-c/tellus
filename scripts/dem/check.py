"""Contrôle des fichiers simulation/data/dem/*.f32 : tailles exactes et statistiques physiques attendues."""

from __future__ import annotations

import sys
from dataclasses import dataclass

import numpy as np
from loguru import logger

from build import CACHE_DIR, LEVELS, OUTPUT_DIR
from centers import load_centers

BODIES = ("earth", "mars", "venus", "moon", "mercury")
EARTH_LAND_TARGET = 0.29
EARTH_LAND_TOLERANCE = 0.03
EARTH_MAX_MIN_M = (5000.0, -8000.0)
MARS_OLYMPUS_MIN_M = 15000.0
PLAUSIBLE_ABS_M = 30000.0


@dataclass(frozen=True)
class Field:
    """Champ d'un corps à un niveau : valeurs et poids d'aire normalisés."""

    values: np.ndarray
    weights: np.ndarray
    centers: np.ndarray


def _cells(level: int) -> int:
    return 10 * 4**level + 2


def _load(body: str, level: int) -> Field:
    target = OUTPUT_DIR / f"{body}_L{level}.f32"
    expected = _cells(level) * 4
    if not target.exists() or target.stat().st_size != expected:
        raise ValueError(f"{target.name} absent ou taille ≠ {expected} octets")
    values = np.fromfile(target, dtype="<f4").astype(np.float64)
    centers, solid_angles = load_centers(CACHE_DIR, level)
    return Field(values, solid_angles / solid_angles.sum(), centers)


def _mean(field: Field, mask: np.ndarray) -> float:
    weights = field.weights * mask
    return float(np.sum(weights * field.values) / np.sum(weights))


def _check_earth(field: Field, level: int) -> list[str]:
    failures: list[str] = []
    land = float(np.sum(field.weights * (field.values > 0.0)))
    if abs(land - EARTH_LAND_TARGET) > EARTH_LAND_TOLERANCE:
        failures.append(f"Terre L{level} : fraction > 0 m = {land:.3f}, attendu 0,29 ± 0,03")
    if field.values.max() <= EARTH_MAX_MIN_M[0] or field.values.min() >= EARTH_MAX_MIN_M[1]:
        failures.append(f"Terre L{level} : extrêmes {field.values.min():.0f} / {field.values.max():.0f} m")
    logger.info(f"Terre L{level} : fraction de terres {land:.3f}")
    return failures


def _check_mars(field: Field, level: int) -> list[str]:
    failures: list[str] = []
    north = _mean(field, field.centers[:, 2] > 0.0)
    south = _mean(field, field.centers[:, 2] < 0.0)
    if not north < south:
        failures.append(f"Mars L{level} : nord {north:.0f} m ≥ sud {south:.0f} m")
    if field.values.max() <= MARS_OLYMPUS_MIN_M:
        failures.append(f"Mars L{level} : maximum {field.values.max():.0f} m (Olympus > 15 000 m attendu)")
    logger.info(f"Mars L{level} : nord {north:.0f} m, sud {south:.0f} m")
    return failures


def _check_generic(body: str, field: Field, level: int) -> list[str]:
    if np.max(np.abs(field.values)) > PLAUSIBLE_ABS_M:
        return [f"{body} L{level} : altitude hors plage plausible"]
    return []


def _check_body(body: str, level: int) -> list[str]:
    field = _load(body, level)
    if not np.all(np.isfinite(field.values)):
        return [f"{body} L{level} : valeurs non finies"]
    logger.info(f"{body} L{level} : min {field.values.min():.0f} m, max {field.values.max():.0f} m")
    if body == "earth":
        return _check_earth(field, level)
    if body == "mars":
        return _check_mars(field, level)
    return _check_generic(body, field, level)


def main() -> int:
    failures: list[str] = []
    for body in BODIES:
        for level in LEVELS:
            try:
                failures.extend(_check_body(body, level))
            except (OSError, ValueError) as exc:
                failures.append(f"{body} L{level} : {exc}")
    for failure in failures:
        logger.error(failure)
    if failures:
        return 1
    logger.info("MNT conformes : tailles et statistiques")
    return 0


if __name__ == "__main__":
    sys.exit(main())
