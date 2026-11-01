"""Pipeline MNT : télécharge les sources, rééchantillonne sur L6 et L7, écrit simulation/data/dem/<corps>_L<n>.f32."""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from loguru import logger

from centers import export_centers, load_centers
from fetch import ensure_raw
from mercury_tiff import mercury_raster
from readers import Raster
from resample import sample_cells
from sources import MERCURY_CACHE_NAME, MERCURY_TIF_URL, SOURCES, SourceSpec, source_for

DEM_DIR = Path(__file__).resolve().parent
REPO_ROOT = DEM_DIR.parent.parent
CACHE_DIR = DEM_DIR / "cache"
OUTPUT_DIR = REPO_ROOT / "simulation" / "data" / "dem"
LEVELS = (6, 7)
BODIES = tuple(spec.body for spec in SOURCES)


def _raster_for(spec: SourceSpec) -> Raster:
    if spec.body == "mercury":
        return mercury_raster(MERCURY_TIF_URL, CACHE_DIR / MERCURY_CACHE_NAME)
    if spec.loader is None:
        raise ValueError(f"aucun lecteur pour {spec.body}")
    return spec.loader(ensure_raw(spec, CACHE_DIR))


def _write_level(body: str, level: int, values: np.ndarray) -> Path:
    if not np.all(np.isfinite(values)):
        raise ValueError(f"{body} L{level} : valeurs non finies après rééchantillonnage")
    target = OUTPUT_DIR / f"{body}_L{level}.f32"
    values.astype("<f4").tofile(target)
    logger.info(f"{target.relative_to(REPO_ROOT)} : {values.size} cellules, {target.stat().st_size} octets")
    return target


def _build_body(body: str) -> None:
    raster = _raster_for(source_for(body))
    for level in LEVELS:
        centers, solid_angles = load_centers(CACHE_DIR, level)
        values = sample_cells(raster, centers, solid_angles)
        _write_level(body, level, values)


def main() -> int:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    export_centers(REPO_ROOT, CACHE_DIR)
    for body in BODIES:
        logger.info(f"corps : {body}")
        _build_body(body)
    return 0


if __name__ == "__main__":
    sys.exit(main())
