#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
if [ -f .pids/app.pid ]; then
  pid="$(cat .pids/app.pid)"
  if kill -0 "$pid" 2>/dev/null; then
    pkill -TERM -P "$pid" 2>/dev/null || true
    kill "$pid" 2>/dev/null || true
    echo "Tellus arrêté (PID $pid)."
  fi
  rm -f .pids/app.pid
else
  echo "Tellus ne tourne pas."
fi
