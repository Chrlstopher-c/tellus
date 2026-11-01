"""Centres de cellules et angles solides, exportés par Rust (test ignoré `export_grid_centers`) : une seule vérité."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import numpy as np
from loguru import logger

CENTER_FLOATS = 4


def export_centers(repo_root: Path, cache_dir: Path) -> None:
    """Relance l'export Rust des grilles L6 et L7 dans `cache_dir` (test ignoré `export_grid_centers`)."""
    env = {**os.environ, "TELLUS_CENTERS_DIR": str(cache_dir)}
    command = ["cargo", "test", "-q", "export_grid_centers", "--", "--ignored"]
    logger.info("export des centres de grille depuis Rust (L6, L7)")
    try:
        subprocess.run(command, cwd=repo_root / "simulation", env=env, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as exc:
        logger.error(f"export des centres échoué : {exc.stderr}")
        raise


def load_centers(cache_dir: Path, level: int) -> tuple[np.ndarray, np.ndarray]:
    """Retourne (centres unitaires (N, 3), angles solides en sr (N,)) pour le niveau donné."""
    raw = np.fromfile(cache_dir / f"grid_L{level}.f64", dtype="<f8")
    table = raw.reshape(-1, CENTER_FLOATS)
    return table[:, :3].copy(), table[:, 3].copy()
