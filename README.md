# Tellus

Une planète entière simulée tick par tick : géologie, climat, cycles de matière, vie qui évolue — puis, à terme,
civilisations. Rien n'est écrit d'avance : tout émerge de règles calquées sur la Terre. Vous observez, et vous
intervenez quand vous le voulez.

## Lancer
```bash
./start.sh     # installe les dépendances si besoin et ouvre l'application
./stop.sh
./restart.sh
```

## Stack
Electron · three.js (WebGPU, repli WebGL2) · React + TypeScript · Rust → WebAssembly (Web Workers) · Vite · pnpm

## Ports
Serveur de développement Vite sur le port défini par `VITE_PORT` (voir `.env.example`).

## Licence
MIT
