# TODO — Tellus

## En cours
- E1 — Squelette qui tourne (voir PROJET.md)

## Backlog (après MVP)
- V2 Civilisations : individus → groupes → normes → institutions → États, tout émergent.
- V3 Technologie : arbre ouvert généré par combinaison de principes (énergie, matériaux, information, biologie),
  sans plafond à 2026.
- Sauvegarde / chargement de mondes, partage d'une graine.
- Sourcer excentricité, obliquité et albédo de Titan et Europe dans planets.json (chargement en `MissingField`).
- Pont Rust : ajouter un logger (entrées non finies ignorées silencieusement dans `clock`).
- Genèse : érosion sans redépôt terrestre ni isostasie post-érosion ; 2 graines sur 10 hors fenêtre de profondeur moyenne (L6).
- Genèse : `tectonics_frozen_mvp` ne teste que le déterminisme (pas de temps dans l'API) — à revoir avec la dérive des plaques.
- Climat : préconditionneur multigrille (L7 à 470-650 ms par pas annuel, plafond 300 itérations atteint).
- Climat : seuil de déglaciation de la boule de neige ×1 000-10 000 CO₂ (littérature ~×350) — ajouter assombrissement de la glace par poussières/cendres.
- Matière : biologie séquentielle (registre partagé) — paralléliser par blocs pour L6 < 40 ms ; coût énergétique de la fixation N non couplé à la NPP.

## Reprise (pause du 2026-11-02)
1. PERF CLIMAT (bloquant) : un pas climat L6 = 1,8-3 s (banc `cargo test --release -- --ignored bench_domains --nocapture`).
   Cause : `climate::hydro::diagnose` (BUCKET_CYCLES × 4 saisons de BiCGSTAB, 400 itér. max, tol 1e-8) relancé à
   chaque pas dès que Δt ≥ 10 ans. Pistes : diagnostic hors du chemin critique (tous les N ticks muraux), tol 1e-5,
   départ à chaud gardé entre appels, moins de cycles de seau. Matière 11-47 ms, vivant 2-3 ms (sans vie).
2. GPU : à la pause, le pilote NVIDIA refusait de nouveaux contextes (« NV_ERR_STATE_IN_USE », 00 h 09) → Electron
   désactive le GPU. Vérifier `nvidia-smi` / redémarrer avant de retester le rendu (e2e app + renderer).
3. Vivant passe 2 (perf saturée ~780 ms/tick, tests longs) : WIP non fusionné dans
   `.claude/worktrees/agent-ae5d91f5f8c97138c` (branche worktree-agent-ae5d91f5f8c97138c), à reprendre à la main.
4. Grandes transitions (E4.S5) : sans elles la vie n'apparaît jamais.
5. Puis E5 : pouvoirs, chronique, deux écrans, sauvegarde, revues design/usage, perf finale.
Travailler en séquence, un seul cargo à la fois (`CARGO_BUILD_JOBS=4`) : les sous-agents parallèles ont fait planter la session.
