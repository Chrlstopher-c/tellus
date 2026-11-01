# Architecture — Tellus

## Domaines
| Dossier | Définition (unique, non chevauchante) |
|---|---|
| `simulation/` | Crate Rust (napi-rs). Toute la vérité du monde : état, règles, ticks. Aucune notion d'affichage. |
| `simulation/src/genesis/` | Création d'une planète à partir d'une graine ou d'un preset réel (plaques, relief, océans). |
| `simulation/src/clock/` | Avancée du temps : ticks, vitesse, ordonnancement des systèmes. |
| `simulation/src/climate/` | Énergie reçue, températures, vents, courants, précipitations, glaces. |
| `simulation/src/matter/` | Cycles de matière (H₂O, C, N, P, O₂) ; seul domaine qui possède les stocks chimiques. |
| `simulation/src/life/flora/` | Types fonctionnels de plantes, feu, biome dominant. |
| `simulation/src/life/fauna/` | Cohortes animales : métabolisme, prédation, dispersion, comportements. |
| `simulation/src/life/evolution/` | Lignées, traits, mutation, spéciation, extinction, phylogénie. |
| `simulation/src/life/transitions/` | Grandes transitions probabilistes (abiogenèse → sortie des eaux). |
| `simulation/src/god/` | Interventions du joueur traduites en perturbations de l'état. |
| `simulation/src/chronicle/` | Détection d'événements marquants depuis l'état (jamais de dates). |
| `simulation/src/shared/` | Grille géodésique, aléa à compteur, unités et constantes physiques, registre de transferts. |
| `simulation/src/bridge/` | Exports napi : commandes, instantanés. Aucune règle du monde. |
| `simulation/data/` | Données de référence (planètes, MNT réduits). |
| `interface/` | Application React (processus de rendu Electron). Lit l'état, ne le calcule jamais. |
| `interface/src/planet-view/` | Globe 3D, carte 2D et bascule entre les deux. |
| `interface/src/time-control/` | Lecture, pause, vitesse, chronologie. |
| `interface/src/inspector/` | Détail d'une région ou d'une espèce, courbes. |
| `interface/src/god-powers/` | Palette d'interventions et leur ciblage sur la planète. |
| `interface/src/sim-bridge/` | Seul point de contact avec la simulation : MessagePort, instantanés, interpolation. |
| `interface/src/shared/` | Design system (tokens, composants de base), journalisation. |
| `desktop/` | Processus principal Electron : fenêtres, mode deux écrans, cycle de vie, utilityProcess de simulation. |
| `mockup/` | Maquette visuelle de référence (HTML/WebGL), hors build. |
| `docs/` | `REGLES.md` (contrat du monde) et `recherche/` (sources). |

## Règles de frontière
- `interface/` n'importe jamais `simulation/` directement : uniquement via `interface/src/sim-bridge/`.
- Un domaine de `simulation/` n'accède à un autre que par son module public (`mod.rs`).
- La simulation est déterministe pour une graine donnée ; tout aléa passe par `shared::rng`.
- Seul `matter/` modifie un stock chimique, et uniquement par `shared::ledger::transfer`.
- `docs/REGLES.md` fait foi pour toute règle du monde.
- Couches futures (civilisations, technologie) = nouveaux domaines de `simulation/`, sans toucher l'existant.

## Flux
utilityProcess (Rust/rayon) avance l'état → instantané (ArrayBuffer transféré) → MessagePort → `sim-bridge`
→ textures de données → `planet-view`. En mode deux écrans, le main relaie le même instantané aux deux fenêtres.
