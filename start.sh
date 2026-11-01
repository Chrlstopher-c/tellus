#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p logs .pids
: > logs/app.log
[ -f .env ] || cp .env.example .env
if [ ! -f package.json ]; then
  echo "Tellus : application pas encore créée (épique E1)." | tee -a logs/app.log
  exit 0
fi
[ -d node_modules ] || pnpm install >> logs/app.log 2>&1
if [ -f .pids/app.pid ] && kill -0 "$(cat .pids/app.pid)" 2>/dev/null; then
  echo "Tellus tourne déjà (PID $(cat .pids/app.pid))."
  exit 0
fi
nohup pnpm dev >> logs/app.log 2>&1 &
echo $! > .pids/app.pid
echo "Tellus démarré (PID $!). Journal : logs/app.log"
