"""Lecteurs des MNT bruts. Chacun rend une grille lat/lon en mètres (référence : niveau de la mer, aréoïde ou rayon moyen)."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np
from loguru import logger

HALF_PIXEL_MOLA_LAT = 0.5 / 16.0
ETOPO5_SHAPE = (2160, 4320)
MOLA_SHAPE = (2880, 5760)
VENUS_SHAPE = (180, 360)


@dataclass(frozen=True)
class Raster:
    """Grille régulière : values[i, j] est au point (first_lat + i·dlat, first_lon + j·dlon)."""

    values: np.ndarray
    first_lat: float
    dlat: float
    first_lon: float
    dlon: float


def _read_array(path: Path, dtype: str, shape: tuple[int, int]) -> np.ndarray:
    try:
        raw = np.fromfile(path, dtype=dtype)
        return raw.reshape(shape).astype(np.float64)
    except (OSError, ValueError) as exc:
        logger.error(f"lecture impossible de {path} (attendu {shape} {dtype}) : {exc}")
        raise


def read_etopo5(path: Path) -> Raster:
    """ETOPO5 NOAA : 5 minutes d'arc, entiers 16 bits gros-boutiste, ligne 0 = pôle Nord, pas de pôle Sud."""
    values = _read_array(path, ">i2", ETOPO5_SHAPE)
    return Raster(values, first_lat=90.0, dlat=-1.0 / 12.0, first_lon=0.0, dlon=1.0 / 12.0)


def read_mola16(path: Path) -> Raster:
    """MOLA MEGDR (MGS) : topographie médiane à 16 pixels/degré, mètres par rapport à l'aréoïde."""
    values = _read_array(path, ">i2", MOLA_SHAPE)
    return Raster(values, first_lat=90.0 - HALF_PIXEL_MOLA_LAT, dlat=-1.0 / 16.0,
                  first_lon=HALF_PIXEL_MOLA_LAT, dlon=1.0 / 16.0)


def read_lola16(path: Path) -> Raster:
    """LOLA LDEM_16 (LRO) : DN × 0,5 m par rapport au rayon moyen lunaire 1 737,4 km."""
    values = _read_array(path, "<i2", MOLA_SHAPE) * 0.5
    return Raster(values, first_lat=90.0 - HALF_PIXEL_MOLA_LAT, dlat=-1.0 / 16.0,
                  first_lon=HALF_PIXEL_MOLA_LAT, dlon=1.0 / 16.0)


def read_venus_grid(path: Path) -> Raster:
    """Magellan (PDS MGN-V-RSS-5) : grille 1°, ASCII en km, longitude de la colonne 0 = 240° E (label AXIS_START)."""
    try:
        values = np.loadtxt(path).reshape(VENUS_SHAPE) * 1000.0
    except (OSError, ValueError) as exc:
        logger.error(f"grille Vénus illisible dans {path} : {exc}")
        raise
    return Raster(values, first_lat=89.5, dlat=-1.0, first_lon=240.0, dlon=1.0)
