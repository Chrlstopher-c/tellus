"""Téléchargement idempotent des MNT bruts dans scripts/dem/cache/, avec vérification SHA-256."""

from __future__ import annotations

import hashlib
import urllib.request
from pathlib import Path

from loguru import logger

from sources import USER_AGENT, SourceSpec

CHUNK_BYTES = 1 << 20
DOWNLOAD_TIMEOUT_S = 600


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(CHUNK_BYTES), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _download(url: str, target: Path) -> None:
    partial = target.with_suffix(target.suffix + ".part")
    logger.info(f"téléchargement {url}")
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=DOWNLOAD_TIMEOUT_S) as response, partial.open("wb") as out:
            for chunk in iter(lambda: response.read(CHUNK_BYTES), b""):
                out.write(chunk)
        partial.replace(target)
    except OSError as exc:
        logger.error(f"échec du téléchargement de {url} : {exc}")
        partial.unlink(missing_ok=True)
        raise


def ensure_raw(spec: SourceSpec, cache_dir: Path) -> Path:
    """Garantit la présence du fichier brut de `spec` et contrôle son empreinte ; retourne son chemin."""
    if spec.url is None or spec.filename is None or spec.sha256 is None:
        raise ValueError(f"{spec.body} n'a pas de fichier brut direct")
    target = cache_dir / spec.filename
    if not target.exists():
        _download(spec.url, target)
    found = sha256_of(target)
    if found != spec.sha256:
        target.unlink()
        raise ValueError(f"empreinte inattendue pour {spec.filename} : {found}")
    return target
