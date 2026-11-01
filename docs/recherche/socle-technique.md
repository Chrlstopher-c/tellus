# Socle technique — Tellus

Recherche réalisée le 2026-11-01, machine : Arch Linux, Hyprland (Wayland), NVIDIA RTX 3060 pilote 615.78.08,
Ryzen 5 2600, deux écrans 1920x1080. Sources : pages officielles, registres npm/crates.io, tests locaux.
Chaque affirmation porte un statut : **[testé]** (exécuté sur cette machine), **[vérifié]** (source officielle lue),
**[secondaire]** (blog ou résultat de recherche, à confirmer), **[non vérifié]** (hypothèse à tester).

## Recommandation

1. **Electron 45.2.1 exact** (dernière stable 45.x, testée). Pas de flag Chromium : WebGPU a fonctionné sans aucun flag [testé].
2. **three@0.187.0** (npm, correspond à la release r187 du 30/10/2026) avec `WebGPURenderer`, shaders TSL. Repli WebGL2
   automatique, mais **sans compute** : le code de simulation doit exister dans un chemin CPU ou dégradé.
3. **Simulation en Rust → wasm (wasm-bindgen 0.2.130, wasm-pack 0.15.1) dans des Web Workers**, mémoire partagée via
   `SharedArrayBuffer`. Toolchain nightly épinglée (date fixée, à re-vérifier). **Pool de workers écrit à la main**,
   sans `wasm-bindgen-rayon` (dernière release 1.3.0 du 21/12/2024 : risque de maintenance).
4. **Un seul propriétaire de la simulation** : une fenêtre renderer dédiée (cachée) héberge les workers et les
   `SharedArrayBuffer`. Chaque fenêtre de vue reçoit des instantanés par `MessagePort`. Un `SharedArrayBuffer` ne traverse
   pas la frontière de processus entre deux fenêtres Electron [non vérifié, connu de la plateforme Chromium].
5. **Napi-rs écarté** comme moteur principal : il rend la simulation native sans contrainte wasm, mais ne peut pas
   toucher le GPU et oblige à copier chaque tick vers le renderer. Garder comme plan B si le wasm multithread est trop lent.
6. **Tests** : `_electron.launch` de Playwright pour l'E2E, WebGPU réel en local (RTX), CI sans GPU limitée aux tests
   unitaires et au chemin WebGL2/CPU.

Risques principaux (détail section 6) : positionnement des fenêtres sous Wayland, divergence des fonctionnalités
entre WebGPU et repli WebGL2, nightly Rust, coût de copie des instantanés vers les vues.

---

## 1. Electron

### Version
- **[vérifié]** Stables au 2026-11-01 : 45.2.1, 44.10.1, 43.8.6. Pré-release 46.0.0-alpha.4, nightly 47.0.0-nightly.
  Source : https://releases.electronjs.org/ (la recherche évoquait 43.2.0 : obsolète, ignorer).
- **[testé]** `pnpm add -D electron@latest` dans un dossier temporaire a installé 45.2.1.
- Cadence annoncée : une nouvelle majeure stable toutes les 8 semaines [secondaire, endoflife.date / isitpatched.com].
  Conséquence : épingler, et monter de version volontairement, avec CI verte avant.

### WebGPU sous Linux / Wayland / NVIDIA
Test réel dans `/tmp/claude-1000/tellus-webgpu-test` (supprimé ensuite), Electron 45.2.1, page servie sur
`http://127.0.0.1` avec `COOP: same-origin` et `COEP: require-corp` :

| Variante (switches) | navigator.gpu | Adaptateur | requestDevice | Compute (dispatch + relecture) | Vulkan | Log notable |
|---|---|---|---|---|---|---|
| aucun flag | oui | NVIDIA Ampere | oui | **oui** | disabled_off | ozone Wayland incompatible avec Vulkan (non bloquant) |
| `enable-unsafe-webgpu` + `enable-features=Vulkan,SharedArrayBuffer` | oui | NVIDIA Ampere | oui | **oui** | enabled_on | idem |
| `enable-unsafe-webgpu` + `enable-features=Vulkan` + `ozone-platform=x11` | oui | **null** | — | — | enabled_on | `GetGeometry failed` ; **aucun adaptateur** |

Conclusions **[testé]** :
- WebGPU est réellement obtenu dans Electron sur cette machine, sans flag.
- Ne pas forcer `ozone-platform=x11` avec Vulkan : l'adaptateur disparaît.
- Le log indique que l'ozone par défaut est Wayland ; Vulkan y est désactivé mais WebGPU fonctionne quand même.
- Une page `data:` n'a pas de `navigator.gpu` (contexte non sécurisé) : servir la page en `http://127.0.0.1` ou via un
  protocole privilégié.
- `screen`/`getGPUFeatureStatus` : `webgpu: "enabled"` dans les trois variantes.
- Non mesuré : débit WebGPU et différence de perf entre les variantes. À faire avec un bench (`pnpm bench`, E3.S5).

Contexte contradictoire : l'issue Electron #41763 (Linux, Electron 29, `requestAdapter()` = null) a été fermée en 2024
sans correctif confirmé, et un utilisateur a signalé le problème en 2025 [vérifié :
https://github.com/electron/electron/issues/41763]. Le test de cette machine (Electron 45, Hyprland, NVIDIA 615) ne le
reproduit pas. Donc : **pas de flag par défaut, détection au démarrage, repli explicite**.

### Flags à retenir
- Par défaut : aucun.
- Si une autre machine n'a pas d'adaptateur : tester `enable-unsafe-webgpu` puis `enable-features=Vulkan`, sans
  `ozone-platform=x11` en même temps que Vulkan [testé].
- Chrome conseille aussi `enable-unsafe-webgpu` et Vulkan sous Linux [vérifié :
  https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips]. Non relu ici.

### Pièges
- **Permissions de contexte** : `navigator.gpu` et `SharedArrayBuffer` exigent un contexte sécurisé et isolé (voir 3.2).
- **Pilotes** : la matrice WebGPU Linux évolue vite. NVIDIA 535.183.01+ est annoncé comme supporté sur Wayland depuis
  Chrome 147 [secondaire, https://github.com/gpuweb/gpuweb/wiki/Implementation-Status]. À re-vérifier à chaque
  montée de version Electron, car Electron suit Chromium avec retard.
- **Firefox** non concerné (cible = Chromium).

## 2. three.js

### Version et renderer
- **[vérifié]** Release courante r187 (30/10/2026), r186 (24/09), r185 (01/07). Source :
  https://github.com/mrdoob/three.js/releases
- **[vérifié]** npm : `three@0.187.0` (dernière version publiée au 2026-11-01).
- **[vérifié]** `WebGPURenderer` utilise le backend WebGPU par défaut et bascule sur WebGL 2 ; `forceWebGL` force
  WebGL 2 : https://threejs.org/manual/en/webgpurenderer
- **[vérifié]** TSL se transpile en WGSL ou GLSL selon le backend (même manuel).
- r187 : pipelines compilés en parallèle (`compileAsync`), `ProxyNodeMaterial`, correctifs de disposal.
  r186 : `compileComputeAsync()`, `updateBefore`/`updateAfter` pour les nœuds de compute, lecture/écriture de storage
  buffers dans des étapes non-compute. r185 : `storageTexture3D` en TSL.

### Compute et repli
- **[secondaire]** Le compute ne tourne pas en mode repli WebGL : sans précaution, le travail disparaît en silence
  (source : skill three.js-claude-skill-package et un blog 2026, à confirmer sur le manuel).
- **Décision** : tout champ calculé en compute doit avoir un chemin de repli explicite (CPU dans le worker, résolution
  réduite ou champ figé). Le repli WebGL2 ne sert qu'au rendu, jamais à la simulation.

### Exemples officiels pertinents (dépôt mrdoob/three.js, dossier `examples/`) [vérifié via GitHub API]
- `webgpu_tsl_earth.html` : globe terrestre en TSL, base pour la vue planète.
- `webgpu_compute_particles.html`, `webgpu_compute_texture_pingpong.html`, `webgpu_compute_water.html`,
  `webgpu_compute_texture.html` : patterns de compute avec textures et storage buffers, utiles pour les champs.
- `webgpu_tsl_compute_attractors_particles.html` : compute TSL sur particules.
- Atmosphère : aucun exemple `atmos*` trouvé dans `examples/` ; à construire à partir du shader TSL du globe.

## 3. Rust → wasm multithread, ou napi-rs

### 3.1 Versions (registres, 2026-11-01)
- wasm-bindgen **0.2.130** (30/10/2026), wasm-pack **0.15.1** (30/10/2026).
- wasm-bindgen-rayon **1.3.0**, dernière release **21/12/2024** [vérifié, crates.io].
- napi **3.16.1** (31/10/2026), napi-derive **3.7.0** (30/10/2026), @napi-rs/cli **3.10.9** (npm).

### 3.2 Multithread wasm : contraintes
- **[vérifié]** wasm-bindgen-rayon demande : toolchain **nightly**, `target-feature=+atomics,+bulk-memory`,
  `--shared-memory`, `--import-memory`, `build-std = ["panic_abort", "std"]`, build `wasm-pack --target web`. Testé
  dans son README avec `nightly-2025-11-15` [vérifié : https://github.com/RReverser/wasm-bindgen-rayon].
- **[non vérifié]** Nightly toujours nécessaire en 2026 : `build-std` reste nightly ; le support des atomics en stable
  est à confirmer sur les notes de release Rust. Épingler une date de nightly dans `rust-toolchain.toml`.
- **[vérifié]** `SharedArrayBuffer` exige un contexte **cross-origin isolated** : `COOP: same-origin` et
  `COEP: require-corp`, et tout sous-contenu doit être compatible [secondaire, tutoriel Cinevva :
  https://app.cinevva.com/tutorials/coop-coep-sharedarraybuffer.html].
- **[testé]** Sur `http://127.0.0.1` servi par Electron avec ces deux en-têtes : `crossOriginIsolated === true`. Le
  chemin de production (protocole custom `app://` avec `protocol.handle` ou `webRequest`) **n'est pas encore testé**.
  L'API `protocol.handle` renvoie un `Response` [vérifié : https://www.electronjs.org/docs/latest/api/session].
- Choix : **pool de workers maison** (quelques dizaines de lignes, contrôle total des tâches et de la mémoire) plutôt
  que `wasm-bindgen-rayon` (dépendance figée depuis 2024).

### 3.3 Comparaison wasm-workers et napi-rs (sans mesure de perf)

| Critère | wasm + Web Workers (renderer) | napi-rs (addon dans le main Node) |
|---|---|---|
| Vitesse de calcul | Bonne, SIMD et threads, sans mesure ici | Natif complet, probablement meilleur |
| Threads | Workers + SAB, pool maison | Threads Rust libres |
| Toolchain | Nightly épinglée (build-std) | Stable |
| Entrée GPU | Direct : workers et WebGPU dans le même renderer | Impossible dans le main : copie vers le renderer à chaque tick |
| Partage mémoire | SAB dans la fenêtre sim | Pas de SAB entre processus : copie ou transfert d'`ArrayBuffer` |
| Headers COOP/COEP | Obligatoires | Non nécessaires |
| Build | `wasm-pack`, un seul artefact | Binaires par plateforme (CI matrix), mais ABI stable : pas de rebuild à chaque version Electron |
| Blocage UI | Non (workers) | Risque si mal isolé du thread principal |
| Déterminisme | Identique (même code Rust) | Identique |
| Débogage | Plus pénible (wasm + workers) | Plus simple |

**Verdict** : wasm dans des workers est le choix cohérent avec la voie retenue (simulation et compute dans le renderer
qui affiche, PROJET E1.S3). Napi est pertinent si le wasm multithread déçoit au bench : coût estimé de copie à chaque
frame à mesurer. Coût de copie indicatif : 160 000 cellules × 32 o × 60 Hz ≈ 300 Mo/s, soit une part notable du budget.

## 4. Mode deux écrans

### API Electron
- **[testé]** `screen.getAllDisplays()` renvoie les deux écrans avec `bounds` en coordonnées logiques :
  - DP-1 : x=2212, y=1505, 1920x1080 (Hyprland : 170 Hz)
  - HDMI-A-1 : x=4132, y=1505, 1920x1080 (Hyprland : 60 Hz)
  - `displayFrequency` vaut **0** sous Wayland : ne pas s'y fier pour le rafraîchissement.
- **[non vérifié]** `BrowserWindow.setPosition` / `setBounds` : sous Wayland, un client ne place pas sa fenêtre.
  Hypothèse à tester : la position est ignorée ou limitée. Solution de repli : règle de fenêtre Hyprland
  (`windowrulev2` avec `monitor` ou `workspace`) ou déplacement par l'utilisateur, puis `setFullScreen(true)`.

### Partage d'état entre fenêtres
- Deux `BrowserWindow` = deux processus renderer. Un `SharedArrayBuffer` ne se partage pas entre eux [non vérifié,
  comportement Chromium connu].
- Modèle retenu : sim renderer unique (propriétaire) → `MessagePort` relayé par le main → vues. Les vues reçoivent des
  `ArrayBuffer` transférés (double buffer, sans copie quand la propriété est transférée).
- `MessageChannelMain` (main ↔ renderer) est l'outil standard d'Electron [vérifié : doc Electron, non relue ici].

## 5. Tests

### Playwright + Electron
- **[vérifié]** `_electron.launch` est documenté (expérimental) ; Electron v21+ supporté ; `executablePath` par défaut
  = binaire du paquet installé ; options `args`, `env`, `cwd` [vérifié : https://playwright.dev/docs/api/class-electron].
- **Version** : `@playwright/test@1.64.1` (npm).
- **[secondaire]** Sur Linux sans écran, Electron en CI demande un écran virtuel (`xvfb-run`).

### WebGPU en test
- **Local** : WebGPU réel (RTX 3060). Les tests E2E WebGPU tournent ici. [testé]
- **CI GitHub (sans GPU)** : WebGPU n'a pas de GPU réel. Le repli logiciel passe par SwiftShader ; un flag
  `--enable-unsafe-swiftshader` est mentionné dans les travaux Electron [secondaire, releases.electronjs.org/pr/53174].
  **[non vérifié]** que cela donne un adaptateur WebGPU fiable sur runner. Les captures canvas WebGPU peuvent sortir noires
  en headless Linux [secondaire].
- **Décision** : CI = unitaires (cargo test, Vitest) + E2E sur le chemin WebGL2/CPU (`forceWebGL`) + tests WebGPU marqués
  et lancés en local. Jamais de test de rendu "validé" sans capture réelle.

## 6. Versions épinglées, architecture de processus, risques

### Versions à épingler (exactes, pas de `^`)
| Paquet | Version | Source |
|---|---|---|
| electron | 45.2.1 | releases.electronjs.org, testé |
| three | 0.187.0 | npm |
| vite | 8.3.5 | npm |
| react | 19.3.1 | npm |
| typescript | 7.0.2 | npm (vérifier la compatibilité des outils avant d'adopter) |
| vitest | 5.0.5 | npm |
| @playwright/test | 1.64.1 | npm |
| wasm-bindgen | 0.2.130 | crates.io |
| wasm-pack | 0.15.1 | crates.io |
| napi / napi-derive | 3.16.1 / 3.7.0 | crates.io (plan B) |
| @napi-rs/cli | 3.10.9 | npm (plan B) |
| Rust nightly | date à fixer (base : nightly-2025-11-15, à remonter) | rust-toolchain.toml |

Gestionnaire : pnpm (`CLAUDE.md`, règle du projet). Pas de `bun install`.

### Architecture de processus
```
main (Electron, Node)           orchestration : cycle de vie, fenêtres, écrans, relais MessagePort
 ├─ sim renderer (caché)        propriétaire de la simulation
 │    ├─ worker × N (wasm)      calcul, SharedArrayBuffer, pool maison
 │    └─ compute GPU (WebGPU)   champs lourds (climat, nutriments)
 ├─ view renderer A (globe)     three WebGPURenderer, lit les instantanés
 └─ view renderer B (carte)     idem, deuxième écran
```
- Une seule source de vérité : `simulation/` (ARCHITECTURE.md). `interface/src/sim-bridge/` est le seul pont.
- Les vues ne calculent jamais l'état (ARCHITECTURE.md, « lit l'état, ne le calcule jamais »).
- `desktop/` ne contient que le main : fenêtres, écrans, relais.

### Risques (classés : probabilité × impact)
1. **Fenêtres Wayland** : positionnement non garanti (probabilité haute, impact moyen). Test sur les deux écrans réels avant
   la E5.S2. Repli : règles Hyprland documentées.
2. **Divergence WebGPU / WebGL2** : compute absent en repli (probabilité haute si un utilisateur n'a pas WebGPU, impact
   élevé). Repli CPU explicite, tests des deux chemins.
3. **Pilotes et Chromium** : WebGPU Linux a déjà cassé (issue #41763). Probabilité moyenne, impact élevé. Épinglage
   Electron, test à chaque montée.
4. **Nightly Rust** : build qui casse à une mise à jour de nightly. Probabilité moyenne, impact moyen. Date fixée.
5. **wasm-bindgen-rayon figé** : dépendance morte. Évité par le pool maison.
6. **Copie des instantanés** vers les vues : coût de bande passante mémoire. À mesurer dès E3.S5. Plan B : napi-rs.
7. **COOP/COEP** : un sous-contenu externe casse l'isolation. Pas de ressource tierce dans l'app.
8. **CI sans GPU** : WebGPU non couvert en CI. Accepté, compensé par les tests locaux.

## Annexe : tests réalisés

Dossier `/tmp/claude-1000/tellus-webgpu-test` (supprimé après test). Fichiers : `package.json`, `main.cjs`.
- Probe : `navigator.gpu`, `requestAdapter()`, `requestDevice()`, dispatch compute 1024 entrées (vérifié : `computeOk`),
  `app.getGPUFeatureStatus()`, `screen.getAllDisplays()`.
- Lancement : binaire `node_modules/.pnpm/electron@45.2.1/node_modules/electron/dist/electron`, `show: false`, timeout 30 s.
- Le dépôt Tellus n'a été modifié que par ce fichier, `docs/recherche/socle-technique.md`.

## Sources

- https://releases.electronjs.org/ (versions Electron)
- https://github.com/electron/electron/issues/41763 (WebGPU Linux, historique)
- https://www.electronjs.org/docs/latest/api/session (protocol.handle)
- https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips (flags Linux)
- https://github.com/gpuweb/gpuweb/wiki/Implementation-Status (statut WebGPU Linux)
- https://github.com/mrdoob/three.js/releases (r185-r187)
- https://threejs.org/manual/en/webgpurenderer (WebGPURenderer, repli WebGL2)
- https://github.com/mrdoob/three.js/tree/dev/examples (exemples WebGPU/TSL, via API)
- https://github.com/RReverser/wasm-bindgen-rayon (prérequis nightly, atomics)
- https://crates.io/crates/wasm-bindgen-rayon (dernière release 21/12/2024)
- https://app.cinevva.com/tutorials/coop-coep-sharedarraybuffer.html (COOP/COEP, secondaire)
- https://playwright.dev/docs/api/class-electron (_electron.launch)
- https://releases.electronjs.org/pr/53174 (SwiftShader, secondaire)
- Registres npm (three, vite, react, typescript, vitest, @playwright/test, @napi-rs/cli) et crates.io
  (wasm-bindgen, wasm-pack, napi, napi-derive) interrogés le 2026-11-01.
