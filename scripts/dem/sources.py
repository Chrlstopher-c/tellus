"""Registre des MNT publics : URL officielle, empreinte SHA-256 et lecteur de chaque corps (voir SOURCES.md)."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from readers import Raster, read_etopo5, read_lola16, read_mola16, read_venus_grid

USER_AGENT = "tellus-dem-pipeline/1.0 (scripts/dem; reproducible build)"
NASA_PDS = "https://pds-geosciences.wustl.edu"
NOAA = "https://www.ngdc.noaa.gov/mgg/global/relief"
USGS = "https://planetarymaps.usgs.gov/mosaic"
MERCURY_TIF_URL = f"{USGS}/Mercury_Messenger_USGS_DEM_Global_665m_v2.tif"
MERCURY_STEP = 16
MERCURY_CACHE_NAME = "mercury_4ppd.npy"


@dataclass(frozen=True)
class SourceSpec:
    """Un MNT brut : fichier téléchargé, empreinte attendue et lecteur (None = traitement spécial)."""

    body: str
    url: str | None
    filename: str | None
    sha256: str | None
    loader: Callable[[Path], Raster] | None


SOURCES: tuple[SourceSpec, ...] = (
    SourceSpec(
        body="earth",
        url=f"{NOAA}/ETOPO5/TOPO/ETOPO5/ETOPO5.DAT",
        filename="etopo5.DAT",
        sha256="471d3dd534144aa9a6551fe3e76320a06a45dade6fd8d45f7d6ad981d59f93c3",
        loader=read_etopo5,
    ),
    SourceSpec(
        body="mars",
        url=f"{NASA_PDS}/mgs/urn-nasa-pds-mgs_mola_topography_derived/meg016/megt90n000eb.img",
        filename="mars_megt16.img",
        sha256="d18d9b9ab8c5516d02e157dd2cde0f1d0d160c21940e953ba22391269a545e7b",
        loader=read_mola16,
    ),
    SourceSpec(
        body="venus",
        url=f"{NASA_PDS}/mgn/mgn-v-rss-5-gravity-l2-v1/mg_5201/topo/topogrd.dat",
        filename="venus_topogrd.dat",
        sha256="76e88aea0d483d6a976975c324573d16c1774f9cb5f84a317c18b8c0d0976010",
        loader=read_venus_grid,
    ),
    SourceSpec(
        body="moon",
        url=f"{NASA_PDS}/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_16.img",
        filename="moon_ldem16.img",
        sha256="a511e40d7a3ea3275945b4da2a1df377133264fab0be94b7434b1cf8907254cb",
        loader=read_lola16,
    ),
    SourceSpec(body="mercury", url=MERCURY_TIF_URL, filename=None, sha256=None, loader=None),
)


def source_for(body: str) -> SourceSpec:
    """Retourne la source d'un corps ; lève KeyError si le corps est inconnu."""
    for spec in SOURCES:
        if spec.body == body:
            return spec
    raise KeyError(body)
