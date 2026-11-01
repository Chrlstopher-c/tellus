# Règles du monde — Tellus (contrat MVP)

Ce document **tranche**. Les détails, équations complètes et sources sont dans `docs/recherche/*.md` ; en cas de
conflit, ce fichier gagne. Chaque règle porte ses tests : ils sont écrits AVANT le code (`cargo test <module>`).
Les valeurs marquées [M] dans les recherches sont à vérifier sur la source avant d'être figées en constante.

## 0. Principes transverses
1. **Rien n'est scripté** : aucun `if tick == …`, aucune date codée, aucun événement prévu. Tout découle de l'état.
2. **Déterminisme** : même graine + mêmes interventions → même état, bit à bit, quel que soit le nombre de threads.
   Aléa par hachage à compteur `rng(graine, tick, cellule, flux)` (SplitMix64), jamais d'état de RNG partagé.
   Réductions parallèles à découpage fixe, combinées dans un ordre fixe.
3. **Conservation** : la matière (C, N, P, O, H₂O) ne change de stock que par `transfer(src, dst, q)`, avec
   `q ← min(q, stock_src)`. Totaux en f64. Écart par tick ≤ 1e-9 relatif, sinon log d'erreur.
4. **Une seule vérité** : le crate `simulation/` calcule ; l'interface affiche et n'invente rien. Le détail visuel
   sous la maille (bruit de relief, texture de forêt) peut nuancer une cellule, jamais contredire sa classe.
5. **Unités SI internes**, conversions seulement à l'affichage. Constantes physiques dans `shared::units`.
6. **Échec bruyant** : NaN, infini, stock négatif ou borne atteinte → log + compteur visible en debug, jamais silencieux.

## 1. Temps (domaine `clock`)
- **Vitesses** : pause · 1 jour/s · 1 an/s · 1 siècle/s · 10 ka/s · 1 Ma/s · 10 Ma/s.
- **Tick découplé de l'image** : la simulation tourne dans son processus à 2-20 ticks/s ; le rendu interpole entre
  deux instantanés à 60 i/s. Durée du tick : `Δt = vitesse / ticks_par_seconde`, bornée par système.
- **Régimes par système** (choisis automatiquement selon Δt) :

| Système | Δt ≤ 1 mois | 1 mois < Δt ≤ 1 an | 1 an < Δt ≤ 1 000 ans | Δt > 1 000 ans |
|---|---|---|---|---|
| Climat | mensuel explicite | 12 sous-pas mensuels | moyenne annuelle | équilibre (relaxation) |
| Matière | mensuel | annuel exact | annuel + super-tick géologique | géologique seul, bio à l'équilibre |
| Vivant | annuel découpé | annuel | écologie à l'équilibre + évolution `p = 1 − e^(−λΔt)` | idem, sous-pas évolutif plafonné |
| Géologie | figée (MVP) | figée | figée | figée |

- **Vitesse effective** : si le processeur ne suit pas, la vitesse réelle baisse et l'interface l'affiche ; aucune
  saute d'état. Test : `clock::effective_speed_reported`.
- **Départ** : planète jeune (océans formés, sans vie) par défaut. Option « laisser mûrir » : la même simulation
  tourne en accéléré maximal sans rendu jusqu'à l'époque choisie. Rien n'est ensemencé.

## 2. Grille (domaine `shared::grid`)
- Icosaèdre géodésique, simulation sur les duaux (hexagones + 12 pentagones). `N = 10·4^L + 2`.
  **L7 = 163 842 cellules (~56 km) par défaut**, L6 = 40 962 en mode léger.
- Voisinage CSR, aires exactes, 3 itérations de Lloyd. Cellules ordonnées sur une courbe de remplissage (localité).
- Pour le rendu : table `texel → (3 cellules, 2 barycentriques)` calculée une fois ; les champs par cellule sont envoyés
  en textures de données, interpolées dans le shader.
- Tests : `grid_cell_count`, `grid_valence` (12 pentagones), `grid_area_sum` (4πR² à 1e-9), `grid_area_uniformity` (< 1,25),
  `texel_lookup_valid` (la cellule la plus proche est toujours un sommet du triangle trouvé).

## 3. Genèse (domaine `genesis`) — détail : `recherche/genese.md`
Pipeline pur G0→G8 : grille → 8-14 plaques (Dijkstra bruité) → rotations d'Euler → frontières (convergente,
divergente, transformante) → profils (fosses, cordillères, dorsales) → âge océanique et profondeur GDH1 → isostasie
d'Airy → fBm + crêtes + érosion (stream power, thermique) → niveau de la mer par quantile d'aire.
- Paramètres de monde : graine, fraction d'eau (défaut 0,708), rayon, gravité, inventaire volatil (eau, CO₂, N₂).
- Tests (tous ceux de `genese.md`, dont) : terres 29,2 % ± 3 ; altitude moyenne des terres 840 ± 150 m ; profondeur
  moyenne 3 700 ± 300 m ; hypsométrie bimodale ; même graine → même hash ; 10 graines → 10 hashs.

## 4. Planètes réelles (domaine `genesis::presets`) — `simulation/data/planets.json`
- Corps jouables : Mercure, Vénus, Terre, Mars, Lune, Titan. Géantes visibles dans le sélecteur mais marquées
  « sans surface », non simulées au MVP.
- Paramètres orbitaux et physiques depuis le JSON (sources NASA). **Relief réel** : MNT publics réduits à la grille
  (Terre ETOPO, Mars MOLA, Vénus Magellan, Lune LOLA) par un script hors ligne reproductible (`scripts/dem/`).
- Effet de serre : profondeur optique grise τ **calibrée par corps** pour retrouver la température de surface observée
  (Terre 288 K, Vénus 737 K, Mars 210 K, Titan 94 K). Pour un monde généré, τ = f(pression, CO₂, H₂O) ajustée sur ces
  trois points. Tests : `presets_teq` (T_eq Terre 255 ± 1 K), `presets_surface_temp` (± 5 K après équilibre).

## 5. Climat (domaine `climate`) — détail : `recherche/climat.md`
1. Insolation journalière moyenne (Berger), orbite képlérienne, table latitude × mois.
2. EBM 2D Budyko–Sellers : `C dT/dt = Q(1−α) − OLR + D∇²T`, diffusion **implicite** (gradient conjugué, ≤ 10 itér.).
   Terre : `OLR = A + B·T − 5,35·ln(CO₂/280)`, A = 210, B = 2,0, D = 0,555. Autres : `OLR = σT⁴/(1 + 3τ/4)`.
3. Capacités : océan (couche de mélange 70 m), terre (10 m équivalent), glace.
4. Albédo : rampe autour de −10 °C sur 10 K (glace/neige 0,62 → 0,30) + albédo de surface par biome.
   Boule de neige possible et hystérétique : c'est voulu.
5. Diagnostics recalculés tous les ~10 ans simulés ou si |ΔT̄| > 0,5 K : vents (cellules Hadley/Ferrel/polaires,
   largeur dépendant de la rotation), évaporation (Clausius–Clapeyron), transport semi-lagrangien de la vapeur,
   précipitations + orographie (pluie au vent, ombre sous le vent), glace de mer.
6. Le CO₂ vient du domaine matière ; le climat ne le possède pas.
- Tests : V-01 à V-30 de `climat.md` (insolation 340 W/m² TOA, T̄ 287 ± 1,5 K, 2×CO₂ = 3,71 W/m², hystérésis de la
  glace, P̄ 1 000 ± 100 mm/an, déserts vers 30°, Köppen ≥ 55 % d'accord). V-14 (Vénus) devient une réussite grâce au τ calibré.

## 6. Matière (domaine `matter`) — détail : `recherche/nutriments.md`
- Boîtes couplées par cellule (sol, végétation, litière, pools RothC, océan de surface) + réservoirs globaux
  (atmosphère, océan profond, sédiments, roches). Éléments : H₂O, C, N, P, O₂.
- Biologie annuelle : NPP de Miami plafond climatique, limitée par Liebig (eau, lumière, N, P) ; respiration Q10 = 2 ;
  décomposition ; fixation d'azote ; échange océan-atmosphère ; pompe biologique.
- Géologie (super-tick 100-1 000 ans) : dégazage volcanique, altération des silicates (thermostat Walker-Hays-Kasting),
  enfouissement organique (seule source nette d'O₂), altération du phosphore.
- Départ : O₂ = 0, CO₂ élevé (inventaire volatil). L'oxygénation n'arrive que si la photosynthèse émerge.
- Intégration exacte des réservoirs linéaires `S' = S·e^(−kΔt) + (I/k)(1 − e^(−kΔt))` → stable à tout Δt.
- Tests : conservation 1e-9 par élément et par tick ; positivité ; stocks stabilisés (atmosphère 600-900 GtC, sols
  1 500-2 400, végétation 450-650) sur une Terre de référence ; pas de dérive > ×10 sur 10 000 ans sans choc.

## 7. Vivant (domaine `life`) — détail : `recherche/vivant.md`
- **Espèce = lignée** (identifiant à chaque scission). Génome = vecteur de 18 traits normalisés (moyenne + variance,
  modèle des moments). Traits 17 (photosynthèse) et 18 (adhérence) à 0 au départ.
- **Cohortes** à la Madingley (≤ 32 par cellule, fusion au-delà), métabolisme MTE `B = B0·M^0,75·e^(−E/kT)`, E = 0,65 eV.
- Prédation Holling III (prédateurs) / II (herbivores), efficacité ~10 %, compromis de traits explicites.
- Flore : 5 types fonctionnels en compétition (Tilman), feu stochastique, biome = classe dominante (Whittaker).
- Dispersion conservative sur le graphe, noyau à queue épaisse. Comportements = règles de flux (fuite, migration,
  chasse en groupe, territorialité), jamais des objets.
- Spéciation allopatrique (isolement ≥ T_iso et distance génétique > θ), extinction (N < N_min pendant T_ext),
  phylogénie enregistrée.
- **Grandes transitions** probabilistes, conditionnées à l'état : abiogenèse (énergie + chimie + surface), photosynthèse,
  eucaryotes (O₂), multicellularité, sortie des eaux. `p = 1 − e^(−λΔt)`. Aucune date.
- Noms d'espèces générés (pseudo-latin déterministe depuis l'identifiant de lignée).
- Tests : les 16 de `vivant.md` (populations bornées, pyramide trophique 0,01-0,3, Kleiber 0,75 ± 0,02, Fisher-KPP,
  biomes ≥ 70 %, spéciation allopatrique, aucun script). Critère d'émergence : sur 50 graines « Terre jeune »,
  la vie apparaît dans 0,5-2 Ga simulés et ne se déclenche jamais sans conditions.

## 8. Interventions (domaine `god`)
Chaque pouvoir est une **perturbation de l'état** passée par les mêmes règles que le reste, journalisée.
| Pouvoir | Effet physique |
|---|---|
| Météorite (taille) | cratère dans le relief, poussières → albédo/aérosols décroissant en années, CO₂, mortalité locale ∝ distance |
| Éruption | CO₂ + aérosols soufrés (refroidissement de quelques années), relief local |
| Forcer le climat | forçage radiatif ΔF ou CO₂ cible, maintenu ou ponctuel |
| Introduire une espèce | nouvelle lignée depuis des traits choisis, dans une cellule |
| Glaciation | baisse temporaire de l'insolation effective |
| Sécheresse | facteur de précipitations régional pendant N ans |
- Tests : `god_meteor_cools` (T̄ baisse puis revient), `god_mass_extinction` (choc → biomasse −50 % en < 500 ans),
  `god_deterministic` (même intervention, même hash).

## 9. Chronologie (domaine `chronicle`)
Détecteurs d'état (jamais de dates) : premiers océans, première vie, photosynthèse, O₂ > 1 %, > 10 %, boule de neige,
extinction de masse, première espèce terrestre, record de taille, intervention. Chaque événement : tick, lieu, cause.

## 10. Rendu (interface `planet-view`) — non délégable
- three.js `WebGPURenderer` + TSL (repli WebGL2 automatique). Les shaders de la maquette (`mockup/shaders/`) sont la
  référence visuelle à porter : champs → couleurs vraies, atmosphère diffusante, nuages advectés par les vents simulés
  et pilotés par les précipitations simulées, spéculaire océanique, glace et neige depuis l'état.
- Globe et carte partagent les mêmes fonctions de couleur : aucune divergence possible. Bascule par morphing du maillage.
- Éclairage d'observation (soleil fixe par rapport à la caméra) au-delà de 1 jour/s ; vrai jour/nuit en dessous.
- Budget : 60 i/s en 1080p sur la machine cible, rendu jamais bloqué par la simulation.

## 11. Performance
- Simulation : ≥ 4 ticks/s à L7 à toutes les vitesses (régime adapté), mesuré par `cargo bench`.
- Rendu : ≥ 60 i/s, temps JS par image < 4 ms, transfert d'instantané < 5 ms.

## 12. Contrat d'intégration entre domaines
Chaque domaine de `simulation/src/<domaine>/` expose la même forme, sans connaître les autres domaines :
```rust
pub struct <Domaine>State { /* SoA : Vec<f32>/Vec<f64> indexés par cellule, + globaux */ }
pub struct <Domaine>Inputs<'a> { /* tranches empruntées produites par d'autres domaines */ }
pub fn init(grid: &Grid, planet: &Planet /* ou paramètres */, ...) -> <Domaine>State;
pub fn step(state: &mut <Domaine>State, grid: &Grid, inputs: &<Domaine>Inputs, dt_years: f64,
            regime: Regime, tick: u64, seed: u64) -> <Domaine>Report;
```
- Les échanges passent uniquement par ces `Inputs` (tranches `&[f32]` par cellule) : aucun domaine n'importe l'état
  interne d'un autre. L'orchestrateur `world/` (écrit après les domaines) compose les appels dans cet ordre par tick :
  `climate → matter → life → god (interventions en file) → chronicle`.
- Champs échangés (noms fixes, unités SI sauf mention) :
  - climat → `temperature_c`, `precip_mm_yr`, `insolation_w_m2`, `ice_fraction`, `wind_u`, `wind_v`, `soil_moisture`(0-1)
  - matière → `co2_ppm` (global), `o2_fraction` (global), `npp_cap_gc_m2_yr`, `n_avail`, `p_avail` (0-1), `nutrient_ocean` (0-1)
  - vivant → `veg_cover` (0-1), `surface_albedo`, `biomass_gc_m2`, `npp_realized_gc_m2_yr`, `photosynth_present` (bool global)
  - genèse → `elevation_m`, `is_ocean`, `crust`, `plate_id`
- `Report` : quelques scalaires (moyennes, stocks totaux, compteurs de clamp) pour les bilans et la chronique.

## 13. Pont v2 (simulation ↔ fenêtres)
Le `utilityProcess` possède le monde et sa boucle ; les fenêtres envoient des commandes et reçoivent des trames.
- **Exports napi** (`simulation/src/bridge/`) :
  `create_world(config_json: String) -> String` (JSON : `{cells, seed, name, lookupWidth, lookupHeight}`),
  `lookup_ids() -> Float32Array` (largeur×hauteur×4, ids en flottants exacts, A = 0),
  `lookup_weights() -> Float32Array` (largeur×hauteur×2),
  `step(dt_years: f64) -> f64` (temps simulé total ; régimes choisis par `clock::regime`),
  `snapshot() -> Float32Array` (cellules×4 : altitude m, température °C, précipitations mm/an, couvert végétal 0-1),
  `vitals() -> String` (JSON : o2Percent, co2Ppm, meanTempC, species, biomassGtC, era),
  `inspect(cell: u32) -> String` (JSON `RegionDetail` de `interface/src/inspector/region.ts`),
  `intervene(json: String) -> String` (`{power, cell, magnitude}` → événement journalisé),
  `drain_events() -> String` (JSON : liste `{tick, years, kind, title, body, cell}` depuis le dernier appel).
- `config_json` = `WorldConfig` de `interface/src/world-setup/world-config.ts` + `level` (défaut 7).
- **Boucle** (sim-process) : `clock::Scheduler` décide de Δt ; à chaque tick : `step`, puis au plus 10 trames/s :
  `{type:'frame', simYears, effectiveRatio, fields, vitals}` ; les événements partent en `{type:'event', …}`.
- Lookup L7 à 2048×1024 : envoyée une seule fois à la création.
