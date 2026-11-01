# Tellus — une planète entière simulée tick par tick, de la roche aux civilisations

## Pourquoi et pour qui
Chris veut regarder un monde vivre, de sa géologie à ses sociétés, en suivant des règles calquées sur la Terre, et y
intervenir quand il le décide. Usage : sessions de contemplation et d'expérimentation sur sa tour.

## Ce que ça apporte
Rien n'est préfait : relief, climat, espèces, puis plus tard lois, pays et technologies émergent de règles simples
empilées. Aucun plafond d'époque (au-delà de 2026 par extrapolation cohérente). Ce n'est la copie d'aucun jeu existant.

## Référence visée
Rendu : visualisations planétaires de la NASA (Blue Marble, Earth Observatory) — atmosphère diffusante, nuages, glaces,
carte à calques lisible. Interface : Apple (épurée, habillée, on s'y sent bien). Simulation : modèle de système Terre
simplifié (EBM, LPJ, Madingley). Référence visuelle validée : `mockup/` (après validation de Chris).

## Voie retenue
- Desktop **Electron 45** (WebGPU confirmé sur la machine, sans flag) ; mode deux écrans par une seconde fenêtre.
- Rendu **three.js 0.187 `WebGPURenderer` + TSL**, repli WebGL2 automatique.
- Simulation **Rust natif (napi-rs) dans un `utilityProcess`**, parallélisée avec rayon. Rust stable, testable avec
  `cargo test`, 2-3× plus rapide que wasm, sans nightly ni COOP/COEP. Instantanés envoyés aux fenêtres par
  `MessagePort` à 2-20 Hz ; le rendu interpole à 60 i/s. Calcul GPU seulement si un banc échoue.
- Tick découplé de l'image, régimes multi-échelles par système (`docs/REGLES.md` § 1).
- Interface React 19 + TypeScript + Framer Motion, Vite, pnpm, pino. Tests : cargo test, Vitest, Playwright Electron.

## Réussite
1. Globe et carte jugés beaux par Chris, sans incohérence visuelle entre les deux vues.
2. ≥ 60 i/s en 1080p à toutes les vitesses ; simulation ≥ 4 ticks/s à 163 842 cellules.
3. Sur une Terre jeune, la vie émerge seule en 0,5-2 Ga simulés ; ensuite les écosystèmes tiennent sans effondrement
   ni explosion absurde (populations bornées, pyramide trophique plausible).

## Raté si
Ça rend moche ou incohérent ; la machine rame ; la vie meurt ou prolifère sans raison physique ; un événement est scripté.

## Hors périmètre (MVP)
Civilisations (V2, prévues dans l'architecture), technologie (V3), tectonique en mouvement, son, multijoueur, cloud.

## Décisions
- 2026-11-01 — Nom : Tellus (la Terre en latin) — court, universel.
- 2026-11-01 — Planète générée par défaut + corps réels du système solaire, vraies données et vrai relief (MNT NASA).
- 2026-11-01 — Un seul écran par défaut ; mode deux écrans activable (globe / carte et panneaux).
- 2026-11-01 — Rôle du joueur : observateur et dieu, sans objectif imposé.
- 2026-11-01 — Simulation déterministe à graine donnée (tests reproductibles, RNG à compteur).
- 2026-11-01 — Licence MIT, dépôt public.
- 2026-11-01 — Temps multi-échelle (1 jour/s → 10 Ma/s) : la vie émerge sur des milliards d'années, rien n'est ensemencé.
- 2026-11-01 — Simulation en Rust natif napi-rs (et non wasm) : stable, plus rapide, pas de nightly — recherche socle.
- 2026-11-01 — Grille géodésique icosaédrique L7 (163 842 cellules) ; tectonique figée au MVP.
- 2026-11-01 — Effet de serre calibré par corps réel (Vénus 737 K atteint) plutôt qu'un modèle gris qui échoue.

## Plan
Règles et tests de chaque story : `docs/REGLES.md`. Visuel = moi ou Opus ; moteur = sous-agents Haiku.

### E1 — Socle qui tourne
- [x] E1.S1 — Electron + Vite + React + TS, fenêtre plein écran, journal pino — VERIFY: `pnpm build && pnpm test:e2e --grep boot`
- [ ] E1.S2 — Crate `simulation` napi-rs chargé dans un utilityProcess, ping/tick — VERIFY: `cargo test && pnpm test:e2e --grep sim-process`
- [x] E1.S3 — Grille géodésique L6/L7 + table texel→cellules — VERIFY: `cargo test grid`
- [ ] E1.S4 — Horloge multi-échelle, vitesses, pause, vitesse effective, instantanés MessagePort — VERIFY: `cargo test clock && pnpm test --run snapshot`
- [ ] E1.S5 — Globe WebGPU/TSL alimenté par un instantané (portage des shaders de la maquette) — VERIFY: `pnpm test:e2e --grep renderer` (≥ 60 i/s mesurés)
- [ ] E1.S6 — start/stop/restart, bancs `cargo bench`, CI verte — VERIFY: `./start.sh && ./stop.sh && gh run list -L1`

### E2 — La planète
- [ ] E2.S1 — Genèse G0-G8 (plaques, frontières, GDH1, isostasie, érosion, niveau de la mer) — VERIFY: `cargo test genesis`
- [ ] E2.S2 — Corps réels : planets.json + MNT réduits (Terre, Mars, Vénus, Lune) — VERIFY: `cargo test presets && ./scripts/dem/check.sh`
- [ ] E2.S3 — Rendu complet du globe : océan, relief, atmosphère, nuages, terminateur — VERIFY: revue design Opus + e2e `globe`
- [ ] E2.S4 — Carte 2D à calques + bascule par morphing, mêmes couleurs — VERIFY: `pnpm test:e2e --grep map-switch`
- [ ] E2.S5 — Sélecteur de monde (graine, fraction d'eau, corps réels), écran de départ — VERIFY: `pnpm test:e2e --grep world-picker`

### E3 — Climat
- [x] E3.S1 — Orbite, insolation Berger, saisons — VERIFY: `cargo test climate::insolation` (V-01 à V-05)
- [ ] E3.S2 — EBM implicite, effet de serre, τ calibrés, régimes de pas — VERIFY: `cargo test climate::ebm climate::planets`
- [ ] E3.S3 — Glace, albédo, boule de neige hystérétique — VERIFY: `cargo test climate::ice`
- [ ] E3.S4 — Vents, évaporation, précipitations, orographie, glace de mer — VERIFY: `cargo test climate::hydro climate::wind climate::koppen`
- [ ] E3.S5 — Nuages, glace et neige rendus depuis l'état simulé ; calques température / pluie — VERIFY: e2e `climate-layers`
- [ ] E3.S6 — Banc climat ≥ 4 ticks/s à L7 toutes vitesses — VERIFY: `cargo bench climate`

### E4 — Matière et vivant
- [ ] E4.S1 — Cycles H₂O, C, N, P, O₂ avec registre de transferts — VERIFY: `cargo test matter` (conservation 1e-9)
- [ ] E4.S2 — Flore : 5 types fonctionnels, feu, biomes émergents — VERIFY: `cargo test life::flora`
- [ ] E4.S3 — Faune : cohortes MTE, Holling, dispersion, comportements — VERIFY: `cargo test life::fauna`
- [ ] E4.S4 — Lignées, mutation, spéciation, extinction, phylogénie — VERIFY: `cargo test life::evolution`
- [ ] E4.S5 — Grandes transitions (abiogenèse → sortie des eaux) sans script — VERIFY: `cargo test life::transitions` (50 graines)
- [ ] E4.S6 — Inspecteur région / espèce, calques biomes et productivité, arbre phylogénétique — VERIFY: `pnpm test:e2e --grep inspector`

### E5 — Dieu, chronologie, deux écrans
- [ ] E5.S1 — Six pouvoirs comme perturbations de l'état — VERIFY: `cargo test god`
- [ ] E5.S2 — Détecteurs d'événements et frise chronologique — VERIFY: `cargo test chronicle && pnpm test:e2e --grep timeline`
- [ ] E5.S3 — Mode deux écrans activable — VERIFY: `pnpm test:e2e --grep dual`
- [ ] E5.S4 — Sauvegarde / chargement local d'un monde — VERIFY: `cargo test save && pnpm test:e2e --grep save`
- [ ] E5.S5 — Boucles de revue design + usage (Opus), corrections, perf finale — VERIFY: e2e complet + 4 formats sans débordement + banc
