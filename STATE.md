# STATE — Tellus

## 2026-11-01 — Session 1 : cadrage
- Découverte faite avec Chris, PROJET.md écrit (vision, voie, plan 5 épiques).
- Dossier, documentation, scripts, CI et dépôt public créés. Aucun code applicatif encore.
- Prochaine étape : E1 (squelette Electron + wasm qui tourne).

## 2026-11-01 — Session 1 (suite) : recherche, règles, maquette
- 6 recherches (Haiku) dans `docs/recherche/` ; contrat tranché dans `docs/REGLES.md`.
- Décisions : simulation Rust natif napi-rs en utilityProcess (pas wasm) ; temps multi-échelle 1 j/s → 10 Ma/s ;
  grille L7 ; effet de serre calibré par corps ; tectonique figée au MVP. WebGPU confirmé dans Electron 45.2.1.
- Maquette `mockup/` (servie par `python3 -m http.server 4317 -d mockup`) : globe, carte, morphing, calques,
  inspecteur, temps, pouvoirs, écran de départ. 67 i/s, 0 erreur console, géométrie OK en 1920×1080 et 1366×768.
- En attente : validation du design par Chris. Socle E1 (grille, squelette Electron) lancé en parallèle.
