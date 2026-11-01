#!/usr/bin/env bash
# Vérifie tailles et statistiques des fichiers simulation/data/dem/*.f32 (après build.sh).
set -euo pipefail

DEM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV="$DEM_DIR/.venv"

if [[ ! -x "$VENV/bin/python" ]]; then
    echo "venv absent : lancer scripts/dem/build.sh d'abord" >&2
    exit 1
fi
"$VENV/bin/python" "$DEM_DIR/check.py"
