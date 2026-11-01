# Tellus — une planète entière simulée tick par tick, de la roche aux civilisations

## Pourquoi et pour qui
Chris veut regarder un monde vivre, de sa géologie à ses sociétés, en suivant des règles calquées sur la Terre, et y
intervenir quand il le décide. Usage : sessions de contemplation et d'expérimentation sur sa tour (deux écrans).

## Ce que ça apporte
Rien n'est préfait : relief, climat, espèces, puis plus tard lois, pays et technologies émergent de règles simples
empilées. Aucun plafond d'époque (au-delà de 2026 par extrapolation cohérente). Ce n'est la copie d'aucun jeu existant.

## Référence visée
Rendu : visualisations planétaires de la NASA (Blue Marble, Earth Observatory) — atmosphère diffusante, nuages, glaces,
lumières de nuit, carte à calques lisible. Simulation : cohérence d'un modèle de système Terre simplifié.

## Voie retenue
- Desktop Electron : Chromium (WebGPU sur Linux/NVIDIA) et fenêtres multiples pour le mode deux écrans.
- Rendu three.js `WebGPURenderer` + shaders TSL (bascule WebGL2 automatique).
- Simulation en Rust compilé en WebAssembly, dans des Web Workers (mémoire partagée, multi-cœurs), champs lourds
  (climat, nutriments) en compute shaders GPU. Le processeur (6 cœurs) est le facteur limitant : le GPU porte le gros.
- Niveaux de détail : agents fins là où l'on regarde, modèles agrégés ailleurs.
- Interface React + TypeScript + Framer Motion, Vite, pnpm. Tests : cargo test, Vitest, Playwright.

## Réussite
1. Globe et carte 2D jugés beaux par Chris, sans incohérence visuelle entre les deux vues.
2. ≥ 60 images/s en 1080p à toutes les vitesses de simulation, tick moyen < 8 ms côté workers.
3. Un écosystème généré tient 10 000 ans simulés sans effondrement ni explosion absurde (populations bornées).

## Raté si
Ça rend moche ou incohérent ; la machine rame ; la vie meurt ou prolifère sans raison physique.

## Hors périmètre (MVP)
Civilisations (prévues dans l'architecture, version 2), son, multijoueur, sauvegarde cloud.

## Décisions
- 2026-11-01 — Nom : Tellus (la Terre en latin) — court, universel.
- 2026-11-01 — Planète générée par défaut + planètes réelles du système solaire avec leurs vraies données.
- 2026-11-01 — Un seul écran par défaut ; mode deux écrans activable (globe / carte et panneaux).
- 2026-11-01 — Rôle du joueur : observateur et dieu, sans objectif imposé.
- 2026-11-01 — Simulation déterministe à graine donnée (tests reproductibles).
- 2026-11-01 — Licence MIT, dépôt public.

## Plan
### E1 — Squelette qui tourne
- [ ] E1.S1 — App Electron + Vite + React, une fenêtre plein écran — VERIFY: `pnpm build && pnpm test:e2e --grep boot`
- [ ] E1.S2 — Détection WebGPU / repli WebGL2 affichée en debug — VERIFY: `pnpm test:e2e --grep renderer`
- [ ] E1.S3 — Crate Rust `simulation` → wasm, appelé depuis un worker — VERIFY: `cargo test && pnpm test --run wasm`
- [ ] E1.S4 — start/stop/restart fonctionnels, CI verte — VERIFY: `./start.sh && ./stop.sh && gh run list -L1`

### E2 — La planète
- [ ] E2.S1 — Génération à graine : plaques, relief, océans (sphère géodésique) — VERIFY: `cargo test genesis`
- [ ] E2.S2 — Globe 3D : océan, relief, atmosphère diffusante, jour/nuit — VERIFY: capture Playwright `globe.png`
- [ ] E2.S3 — Carte 2D (projection équivalente) à calques, mêmes couleurs que le globe — VERIFY: `pnpm test:e2e --grep map`
- [ ] E2.S4 — Bascule globe ↔ carte animée, caméra orbitale et zoom — VERIFY: `pnpm test:e2e --grep switch`
- [ ] E2.S5 — Planètes réelles (Mercure → Neptune) depuis données physiques — VERIFY: `cargo test presets`

### E3 — Horloge et climat
- [ ] E3.S1 — Moteur de ticks, pause → millénaires/minute — VERIFY: `cargo test clock && pnpm test --run speed`
- [ ] E3.S2 — Insolation, saisons, températures par cellule — VERIFY: `cargo test climate::insolation`
- [ ] E3.S3 — Vents, courants, précipitations, glaces (compute GPU) — VERIFY: `pnpm test --run climate-gpu`
- [ ] E3.S4 — Nuages et glaces rendus depuis l'état simulé — VERIFY: capture Playwright `climate.png`
- [ ] E3.S5 — Budget perf tenu à vitesse max — VERIFY: `pnpm bench` (≥ 60 fps, tick < 8 ms)

### E4 — Le vivant
- [ ] E4.S1 — Cycles eau, carbone, azote, phosphore — VERIFY: `cargo test nutrients` (conservation de masse)
- [ ] E4.S2 — Végétation et biomes émergents du climat — VERIFY: `cargo test life::flora`
- [ ] E4.S3 — Espèces à génome, comportements, chaîne alimentaire — VERIFY: `cargo test life::fauna`
- [ ] E4.S4 — Mutation, sélection, spéciation, extinction — VERIFY: `cargo test evolution` (10 000 ans bornés)
- [ ] E4.S5 — Inspecteur : cliquer une région / espèce, courbes de population — VERIFY: `pnpm test:e2e --grep inspector`

### E5 — Dieu et deux écrans
- [ ] E5.S1 — Interventions : météorite, volcan, climat, introduction d'espèce — VERIFY: `cargo test god`
- [ ] E5.S2 — Mode deux écrans activable (globe / carte et panneaux) — VERIFY: `pnpm test:e2e --grep dual`
- [ ] E5.S3 — Chronologie des événements marquants — VERIFY: `pnpm test:e2e --grep timeline`
- [ ] E5.S4 — Revues design et usage (Opus), corrections — VERIFY: e2e complet + 4 formats sans débordement
