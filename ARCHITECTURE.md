# Architecture — Tellus

## Domaines
| Dossier | Définition (unique, non chevauchante) |
|---|---|
| `simulation/` | Crate Rust → wasm. Toute la vérité du monde : état, règles, ticks. Aucune notion d'affichage. |
| `simulation/src/genesis/` | Création d'une planète à partir d'une graine ou d'un preset réel (plaques, relief, océans). |
| `simulation/src/clock/` | Avancée du temps : ticks, vitesse, ordonnancement des systèmes. |
| `simulation/src/climate/` | Énergie reçue, températures, vents, courants, précipitations, glaces. |
| `simulation/src/nutrients/` | Cycles de matière (eau, C, N, P) avec conservation de masse. |
| `simulation/src/life/` | Flore, faune, génomes, comportements, évolution. |
| `simulation/src/god/` | Interventions du joueur traduites en perturbations de l'état. |
| `simulation/src/shared/` | Grille géodésique, aléa déterministe, unités physiques. |
| `interface/` | Application React (processus de rendu Electron). Lit l'état, ne le calcule jamais. |
| `interface/src/planet-view/` | Globe 3D, carte 2D et bascule entre les deux. |
| `interface/src/time-control/` | Lecture, pause, vitesse, chronologie. |
| `interface/src/inspector/` | Détail d'une région ou d'une espèce, courbes. |
| `interface/src/god-powers/` | Palette d'interventions et leur ciblage sur la planète. |
| `interface/src/sim-bridge/` | Seul point de contact avec `simulation/` : workers, wasm, tampons partagés. |
| `interface/src/shared/` | Design system (tokens, composants de base), journalisation. |
| `desktop/` | Processus principal Electron : fenêtres, mode deux écrans, cycle de vie. |

## Règles de frontière
- `interface/` n'importe jamais `simulation/` directement : uniquement via `interface/src/sim-bridge/`.
- Un domaine de `simulation/` n'accède à un autre que par son module public (`mod.rs`).
- La simulation est déterministe pour une graine donnée ; tout aléa passe par `shared::rng`.
- Couches futures (civilisations, technologie) = nouveaux domaines de `simulation/`, sans toucher l'existant.

## Flux
Workers (wasm) avancent l'état → instantanés en `SharedArrayBuffer` → `sim-bridge` → textures GPU → `planet-view`.
