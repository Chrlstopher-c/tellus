"""Moyenne d'un MNT par cellule : K sous-points équi-surfaciques dans la calotte de chaque cellule, interpolation bilinéaire."""

from __future__ import annotations

import numpy as np

from readers import Raster

SUBSAMPLES = 64
CHUNK_CELLS = 4096
GOLDEN_ANGLE = np.pi * (3.0 - np.sqrt(5.0))
POLE_GUARD = 0.9


def _tangent_basis(centers: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    helper = np.zeros_like(centers)
    use_x = np.abs(centers[:, 2]) > POLE_GUARD
    helper[~use_x, 2] = 1.0
    helper[use_x, 0] = 1.0
    east = np.cross(centers, helper)
    east /= np.linalg.norm(east, axis=1, keepdims=True)
    north = np.cross(centers, east)
    return east, north


def _cap_points(centers: np.ndarray, solid_angles: np.ndarray) -> np.ndarray:
    """Points (m, K, 3) répartis uniformément en surface dans la calotte de chaque cellule (Archimède)."""
    k = np.arange(SUBSAMPLES, dtype=np.float64)
    fraction = (k + 0.5) / SUBSAMPLES
    cos_theta = 1.0 - fraction[None, :] * (solid_angles / (2.0 * np.pi))[:, None]
    sin_theta = np.sqrt(np.clip(1.0 - cos_theta * cos_theta, 0.0, None))
    phi = k * GOLDEN_ANGLE
    east, north = _tangent_basis(centers)
    ring = np.cos(phi)[None, :, None] * east[:, None, :] + np.sin(phi)[None, :, None] * north[:, None, :]
    return cos_theta[..., None] * centers[:, None, :] + sin_theta[..., None] * ring


def _to_latlon(points: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    lat = np.degrees(np.arcsin(np.clip(points[..., 2], -1.0, 1.0)))
    lon = np.degrees(np.arctan2(points[..., 1], points[..., 0]))
    return lat, lon


def _bilinear(raster: Raster, lat: np.ndarray, lon: np.ndarray) -> np.ndarray:
    values = raster.values
    nrows, ncols = values.shape
    fr = np.clip((lat - raster.first_lat) / raster.dlat, 0.0, nrows - 1.0)
    fc = np.mod((lon - raster.first_lon) / raster.dlon, ncols)
    r0 = np.floor(fr).astype(np.int64)
    r1 = np.minimum(r0 + 1, nrows - 1)
    c0 = np.floor(fc).astype(np.int64) % ncols
    c1 = (c0 + 1) % ncols
    wr = fr - r0
    wc = fc - np.floor(fc)
    top = values[r0, c0] * (1.0 - wc) + values[r0, c1] * wc
    bottom = values[r1, c0] * (1.0 - wc) + values[r1, c1] * wc
    return top * (1.0 - wr) + bottom * wr


def sample_cells(raster: Raster, centers: np.ndarray, solid_angles: np.ndarray) -> np.ndarray:
    """Altitude moyenne (m) de chaque cellule ; NaN si le MNT n'a aucune donnée valide dans la cellule."""
    out = np.empty(centers.shape[0], dtype=np.float64)
    for start in range(0, centers.shape[0], CHUNK_CELLS):
        stop = min(start + CHUNK_CELLS, centers.shape[0])
        points = _cap_points(centers[start:stop], solid_angles[start:stop])
        lat, lon = _to_latlon(points)
        out[start:stop] = np.nanmean(_bilinear(raster, lat, lon), axis=1)
    return out
