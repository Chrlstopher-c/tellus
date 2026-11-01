"""DEM global de Mercure (USGS, GeoTIFF 665 m, 531 Mo) lu par requêtes HTTP partielles : une ligne sur 16, une colonne sur 16.

Le fichier est non compressé, une bande par ligne (strip) : chaque ligne se lit par un seul intervalle d'octets.
Le téléchargement complet (531 Mo) dépasse la limite de 50 Mo ; on ne récupère que les lignes utiles (≈ 33 Mo).
"""

from __future__ import annotations

import re
import struct
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
from loguru import logger

from readers import Raster
from sources import MERCURY_STEP, USER_AGENT

HEADER_BYTES = 131_072
TAG_WIDTH, TAG_HEIGHT, TAG_BITS, TAG_COMPRESSION = 256, 257, 258, 259
TAG_STRIP_OFFSETS, TAG_ROWS_PER_STRIP, TAG_STRIP_BYTES, TAG_SAMPLE_FORMAT = 273, 278, 279, 339
TAG_GDAL_METADATA = 42112
TYPE_SIZES = {1: 1, 2: 1, 3: 2, 4: 4, 5: 8, 12: 8}
TYPE_CODES = {1: "B", 2: "c", 3: "H", 4: "I", 5: "II", 12: "d"}
LITTLE_ENDIAN = b"II"
PROGRESS_EVERY = 120
FETCH_WORKERS = 8


def _http_range(url: str, start: int, end: int) -> bytes:
    request = urllib.request.Request(url, headers={"Range": f"bytes={start}-{end}", "User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            return response.read()
    except OSError as exc:
        logger.error(f"requête partielle échouée ({start}-{end}) sur {url} : {exc}")
        raise


def _tag_values(head: bytes, ifd: int) -> dict[int, tuple[int, int, bytes]]:
    count = struct.unpack_from("<H", head, ifd)[0]
    tags: dict[int, tuple[int, int, bytes]] = {}
    for entry in range(count):
        tag, kind, n = struct.unpack_from("<HHI", head, ifd + 2 + 12 * entry)
        raw = head[ifd + 10 + 12 * entry: ifd + 14 + 12 * entry]
        size = TYPE_SIZES[kind] * n
        if size <= 4:
            tags[tag] = (kind, n, raw[:size])
        else:
            pointer = struct.unpack("<I", raw)[0]
            tags[tag] = (kind, n, head[pointer: pointer + size])
    return tags


def _ints(tags: dict[int, tuple[int, int, bytes]], tag: int) -> list[int]:
    kind, n, blob = tags[tag]
    fmt = "<" + TYPE_CODES[kind] * n
    return list(struct.unpack(fmt, blob))


def _scale_from_metadata(tags: dict[int, tuple[int, int, bytes]]) -> float:
    if TAG_GDAL_METADATA not in tags:
        return 1.0
    text = tags[TAG_GDAL_METADATA][2].decode("latin-1")
    match = re.search(r'name="SCALE"[^>]*>([0-9.eE+-]+)<', text)
    return float(match.group(1)) if match else 1.0


def _read_layout(url: str) -> tuple[int, int, int, list[int], float]:
    head = _http_range(url, 0, HEADER_BYTES - 1)
    if head[:2] != LITTLE_ENDIAN or struct.unpack_from("<H", head, 2)[0] != 42:
        raise ValueError("TIFF inattendu : seul le little-endian classique est géré")
    tags = _tag_values(head, struct.unpack_from("<I", head, 4)[0])
    width, height = _ints(tags, TAG_WIDTH)[0], _ints(tags, TAG_HEIGHT)[0]
    if _ints(tags, TAG_BITS)[0] != 16 or _ints(tags, TAG_COMPRESSION)[0] != 1:
        raise ValueError("TIFF non supporté : 16 bits non compressé requis")
    if _ints(tags, TAG_SAMPLE_FORMAT)[0] != 2 or _ints(tags, TAG_ROWS_PER_STRIP)[0] != 1:
        raise ValueError("TIFF non supporté : entier signé, une ligne par bande requis")
    offsets = _ints(tags, TAG_STRIP_OFFSETS)
    return width, height, len(offsets), offsets, _scale_from_metadata(tags)


def _fetch_row(url: str, offsets: list[int], width: int, k: int) -> np.ndarray:
    cols = np.arange(width // MERCURY_STEP) * MERCURY_STEP + MERCURY_STEP // 2
    line = k * MERCURY_STEP + MERCURY_STEP // 2
    raw = _http_range(url, offsets[line], offsets[line] + 2 * width - 1)
    return np.frombuffer(raw, dtype="<i2")[cols].astype(np.float64)


def _fetch_rows(url: str, offsets: list[int], width: int, height: int) -> np.ndarray:
    count = height // MERCURY_STEP
    rows = np.empty((count, width // MERCURY_STEP), dtype=np.float64)
    with ThreadPoolExecutor(max_workers=FETCH_WORKERS) as pool:
        futures = [pool.submit(_fetch_row, url, offsets, width, k) for k in range(count)]
        for k, future in enumerate(futures):
            rows[k] = future.result()
            if k % PROGRESS_EVERY == 0:
                logger.info(f"Mercure : ligne {k}/{count}")
    return rows


def decimate_mercury(url: str, cache_file: Path) -> np.ndarray:
    """Retourne le DEM de Mercure à 4 pixels/degré (mètres, NaN = pas de donnée), mis en cache dans `cache_file`."""
    if cache_file.exists():
        return np.load(cache_file)
    width, height, strips, offsets, scale = _read_layout(url)
    if strips != height:
        raise ValueError(f"nombre de bandes {strips} différent de la hauteur {height}")
    logger.info(f"Mercure : {width}x{height} px, facteur {scale} m/DN, décimation 1/{MERCURY_STEP}")
    dn = _fetch_rows(url, offsets, width, height)
    metres = np.where(dn == -32768, np.nan, dn * scale)
    np.save(cache_file, metres)
    return metres


def mercury_raster(url: str, cache_file: Path) -> Raster:
    """Grille Mercure à 4 pixels/degré, cellule centrée, longitude -180 à 180."""
    values = decimate_mercury(url, cache_file)
    return Raster(values, first_lat=90.0 - 0.125, dlat=-0.25, first_lon=-180.0 + 0.125, dlon=0.25)
