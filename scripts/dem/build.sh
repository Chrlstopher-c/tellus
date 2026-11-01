#!/usr/bin/env bash
# Reconstruit simulation/data/dem/*.f32 depuis les MNT publics (voir SOURCES.md). Venv Python local, jamais global.
set -euo pipefail

DEM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV="$DEM_DIR/.venv"

if [[ ! -x "$VENV/bin/python" ]]; then
    python3 -m venv "$VENV"
fi
"$VENV/bin/pip" install --quiet -r "$DEM_DIR/requirements.txt"
"$VENV/bin/python" "$DEM_DIR/build.py"
