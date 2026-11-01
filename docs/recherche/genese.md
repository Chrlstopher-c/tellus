# Genèse procédurale de Tellus : recherche documentaire

Date : 2026-11-01. Périmètre : génération d'une planète à partir d'une graine, puis évolution lente.
Légende : **[vérifié]** = confirmé par une page ou un résumé lu ; **[mémoire]** = à vérifier avant implémentation.
Le corps de l'article Cortial et al. 2019 n'a pas pu être lu (PDF bloqué, HTTP 403) : sa méthode est résumée à partir de son résumé et de sources secondaires.

## Recommandation

- **Grille** : icosaèdre subdivisé géodésique (triangles), simulation sur les duaux (hexagones, 12 pentagones). Table de voisinage CSR, aucune singularité polaire.
- **Niveaux** : L6 = 40 962 cellules (~112 km), L7 = 163 842 (~56 km, défaut), L8 = 655 362 (~28 km, champs GPU seulement, optionnel).
- **Budget** : L7 sur 4 à 5 workers wasm ; champs lourds (climat, nutriments) en compute GPU. Tick < 8 ms à mesurer (E3.S5).
- **Genèse** : 8 à 14 plaques (graines, croissance pondérée), rotation d'Euler par plaque, frontières classées par vitesse relative, élévation = profils de frontière + âge océanique (GDH1) + isostasie d'Airy + bruit fBm.
- **Niveau de la mer** : quantile pondéré par aire, pour une fraction d'eau de 0,708 ; amplitude du relief calée sur l'altitude moyenne des terres.
- **Tectonique** : figée au MVP. À 5 cm/an, une plaque bouge de 0,5 km en 10 000 ans : sans effet sur les échelles de vie du PROJET.
- **Test d'entrée** : graine fixe → hash identique ; tous les tests de la dernière section passent.

---

## 1. Grille sphérique

| Critère | Icosaèdre géodésique (dual) | Cube-sphère équiangulaire | Fibonacci | HEALPix |
|---|---|---|---|---|
| Uniformité des aires | max/min ≈ 1,2 brut, meilleur après relaxation **[mémoire]** | max/min ≈ 1,3 (équiangulaire) ; ≈ 1,7 pour gnomonique équidistante **[mémoire]** | quasi uniforme, aires non exactes ; RMSE de calotte −40 % vs lat-lon **[vérifié, González 2010]** | aire exactement égale **[vérifié, Górski 2005]** |
| Voisinage | 6 (12 cellules à 5) | 4 ; 8 coins à valence 3 ; raccords entre faces | aucun : k-d tree requis | 8, singularités aux coins de base |
| Indexation GPU | table CSR générique | (face, i, j) trivial | calcul à la volée | bits (nested) ; ring non trivial |
| Vers texture équirectangulaire | bake par triangle, une fois | cubemap natif (three.js) | conversion coûteuse | bits, nested seulement |
| Cellules (N) | 40 962 / 163 842 / 655 362 | 6·82² / 6·164² / 6·329² | arbitraire | 49 152 / 196 608 / 786 432 (puissances de 2 ; nested) |

Espacement moyen : sqrt(4πR²/N), R = 6 371 km. Icosaèdre : 111,6 / 55,8 / 27,9 km **[calcul]**.

**Choix : icosaèdre géodésique, duaux hexagonaux.** Justifications :
1. Valence 6 presque partout : stencils de diffusion et de flux identiques pour toutes les cellules, sans cas polaire.
2. Aires les plus uniformes des grilles qui gardent des voisinages réguliers, après une relaxation de Lloyd sur la sphère.
3. Tout passe par une table CSR : le code de simulation est indépendant de la géométrie ; seul le bake de texture dépend de la forme.
4. Les modèles de climat à maillage icosaédrique (ICON, DWD) suivent cette voie **[mémoire]**.

La cube-sphère est le plan B si le bake GPU devient le goulot : indexation plus simple, cubemap natif.

Niveaux :

| Niveau | Cellules | Espacement | Usage | Budget |
|---|---|---|---|---|
| L6 | 40 962 | ~112 km | tests unitaires, machines faibles | tick < 2 ms |
| L7 | 163 842 | ~56 km | simulation par défaut | tick < 8 ms (4 à 5 workers) |
| L8 | 655 362 | ~28 km | champs climatiques GPU, bake de rendu | < 4 ms GPU **[estimation]** |

Estimation CPU L7 : ~20 opérations scalaires × 6 voisins × 163 842 cellules ≈ 2·10⁷ opérations par champ ; en wasm
(≈ 2 à 3 × le natif) et sur 4 workers, 2 à 6 ms **[estimation, à mesurer]**.

Génération du maillage : subdivision récursive d'un icosaèdre, sommets normalisés, duaux par centres des triangles.
Aire d'une cellule = un tiers de chaque triangle incident (exact pour le dual). Complexité O(N).

## 2. Tectonique procédurale

### 2.1 Références

- **Cortial, Peytavie, Galin, Guérin, « Procedural Tectonic Planets », CGF 38(2), 2019** [vérifié résumé : https://diglib.eg.org/handle/10.1111/cgf13614 ; DOI 10.1111/cgf.13614]. Résumé : plaques pilotables par l'utilisateur, subduction et collisions déformant croûtes continentale et océanique, dorsales, chaînes, arcs insulaires, rifts ; détail final par bruit ou données réelles. Suite : « Real-Time Hyper-Amplification of Planets », The Visual Computer, 2020 [vérifié, titre].
- **Blog math33, gamedev.net, « how i try to create entire scientifically realistic worlds »** [vérifié résumé : https://gamedev.net/blogs/entry/2262882-how-i-try-to-create-entire-scientifically-realistic-worlds-with-math33] : graines aléatoires ; type de frontière déduit de la comparaison entre le mouvement de la plaque et la direction vers le voisin.
- **PlaTec** et **Experilous** : non trouvés dans la recherche ; non utilisés comme sources.

Ce que la méthode Cortial implique pour nous (à vérifier dans le PDF) : contrôle utilisateur des mouvements,
profils de frontière, pas de simulation physique du manteau. Le choix retenu ci-dessous suit cette logique.

### 2.2 Plaques

Graines : N_p points (8 à 14) tirés par le RNG seedé, puis Lloyd sur la sphère (quelques itérations).
Croissance : Dijkstra multi-sources sur le graphe des cellules, distance = longueur d'arc × (1 + bruit fBm basse fréquence),
pour des frontières irrégulières. Complexité O(N log N).
Type de croûte par plaque : océanique ou continentale, tirage à 50 % avec au moins une plaque de chaque type.

### 2.3 Mouvement (rotation d'Euler)

Chaque plaque p : axe unitaire u_p et vitesse angulaire ω_p. Vitesse en x (rayon R) : **v_p(x) = R · ω_p · (u_p × x̂)**.
Ordre de grandeur : 1 cm/an ≈ 10 km/Ma ≈ 1,57·10⁻³ rad/Ma **[calcul]**. Plage cible : 1 à 15 cm/an.

### 2.4 Classification des frontières

Pour une arête (a ∈ P, b ∈ Q), direction n̂ de a vers b projetée sur le plan tangent, Δv = v_P(x) − v_Q(x) :
- convergence c = −Δv · n̂ ; cisaillement s = |Δv − (Δv · n̂) n̂|
- **c > s** → convergente ; **−c > s** → divergente ; sinon **transformante**.

| Type | Océan–océan | Océan–continent | Continent–continent |
|---|---|---|---|
| Convergente | arc insulaire (+1 à +2 km), fosse (−6 à −8 km) | fosse (−6 à −8 km, σ ≈ 50 km) + cordillère côté continent (+2,5 à +3,5 km, σ ≈ 150 km) | chaîne de collision (+3 à +5 km, σ ≈ 300 km) |
| Divergente | dorsale (profondeur = GDH1 de l'âge 0) | rift puis dorsale | rift (cuvette −0,5 à −1 km) |
| Transformante | faille linéaire, relief négligeable | idem | idem |

Profil de frontière : h(d) = A · exp(−d² / 2σ²), d = distance à la frontière (Dijkstra depuis les arêtes classées).
Valeurs A, σ = indicatives **[mémoire]**, à caler sur le rendu.

### 2.5 Croûte et profondeur océanique

- Continentale : ρ ≈ 2,7 g/cm³, épaisseur ≈ 35 km **[mémoire]**.
- Océanique : ρ ≈ 3,0 g/cm³, épaisseur ≈ 7 km **[mémoire]**.
- Âge océanique : t(x) = d_dorsale(x) / v_demi-ouverture (distance à la dorsale la plus proche, divisée par la vitesse
  d'écartement de la moitié de la plaque).
- Profondeur : **GDH1 (Stein & Stein 1992)** [vérifié, formules GMT :
  https://docs.generic-mapping-tools.org/6.1/gallery/ex49.html]
  - d(t) = 2600 + 365·√t (t ≤ 20 Ma)
  - d(t) = 5651 − 2473·exp(−0,0278·t) (t > 20 Ma)
  - Continuité à 20 Ma : 4232 m des deux côtés (vérifié par calcul). À 100 Ma : 5498 m.
  - Forme demandée 2500 + 350·√t : valable jusqu'à ~70 Ma (Parsons–Sclater, GMT ex. 49). Choix retenu : GDH1.
  - Source secondaire : https://en.wikipedia.org/wiki/Seafloor_depth_versus_age (formules du modèle de demi-espace :
    Davis & Lister 1974 ; Parsons & Sclater 1977 : d = 6400 − 3200·exp(−t/62,8)).
- Isostasie d'Airy (continents) [mémoire] : h = (T − T_ref) · (ρ_m − ρ_c) / ρ_m, avec T_ref = 35 km, ρ_m = 3,3 g/cm³.
  Soit 1 km de racine crustale ≈ 180 m de relief ; autrement dit r = 4,5 · h. Exemple : T = 45 km → h = 1 818 m.

## 3. Relief fin

- **fBm** [mémoire, forme classique] : h(x) = Σ_{k=0}^{O−1} a^k · n(f^k · x̂ + décalage), a = 0,5, f = 2, O = 6 à 8.
  Bruit 3D évalué sur le vecteur unitaire x̂ : pas de couture, pas de pôle.
- **Ridged** (orogènes seulement) : r = (1 − |n|)², pondéré par le signal des octaves précédentes.
- **Amplitude** : 1 à 3 km sur les continents ; masque réduit à 0 au-dessus des océans profonds.
- **Résolution** : à L7 une cellule fait ~56 km, donc le bruit fin est évalué au rendu (shader) à partir de la graine,
  ou bâké à L9 (2,6 M cellules, ~14 km) une fois, à la génèse.
- **Érosion hydraulique** (stream power) [vérifié, principe] : ∂z/∂t = U − K·A^m·S^n, A = aire drainée, S = pente, m ≈ 0,5, n ≈ 1.
  Solution implicite O(n) : Braun & Willett 2013, *Geomorphology* 180–181, 170–179
  [vérifié : https://landlab.readthedocs.io/en/latest/reference/components/stream_power.html].
  Ordre topologique (flux D8), accumulation, puis résolution implicite. Itérations : 100 à 200 à la génèse, quelques secondes.
- **Érosion thermique** : si pente > talus (~35°), transfert de 50 % de l'excès vers les voisins plus bas. Simple, O(N).
- **Érosion visuelle alternative** : Beneš & Forsbach 2002, quatre étapes indépendantes (érosion, transport, dépôt,
  évaporation) [vérifié : https://www.cs.purdue.edu/cgvlab/www/resources/papers/Benes-2002-Visual_simulation_of_hydraulic_erosion.pdf].
- **Isostasie** : recalculée après érosion. Contrôle : la masse érodée est égale à la masse déposée (Σ A·Δh = 0).

## 4. Niveau de la mer et validation

Cibles de la Terre :

| Grandeur | Valeur | Source |
|---|---|---|
| Fraction de terres | ≈ 29,2 % (océan ≈ 70,8 %) | [mémoire, cohérent avec la recherche] |
| Altitude moyenne des terres | ≈ 840 m | [vérifié, un cours universitaire ; mémoire NOAA] |
| Profondeur moyenne des océans | 3 682 m (Charette & Smith 2010) ; ≈ 3 688 m (NOAA ETOPO1, mémoire) | [vérifié partiel] |
| Forme | bimodale : pic continental vers +100 m, pic océanique vers −4 700 m | [vérifié, Britannica : https://www.britannica.com/print/article/280163] |

Un cours cité donne 3 800 m pour la profondeur moyenne : écart retenu comme erreur de source. Cible : 3 700 ± 50 m.

**Calibration** (deux paramètres) :
1. **Niveau de la mer s** : quantile pondéré par aire. Trier h, cumuler les aires, prendre s tel que
   Σ_{h_i < s} A_i / Σ A_i = f_eau (0,708). Coût O(N log N), ~20 ms à L7 **[estimation]**. Puis h' = h − s.
2. **Facteur d'amplitude α** : multiplier les profils de frontière et le fBm continental pour que
   Σ_{h'>0} A·h' / Σ_{h'>0} A = 840 m. Puis vérifier la profondeur moyenne océanique.

Si l'histogramme n'est pas bimodal (pic unique), revoir l'amplitude du bruit et l'isostasie avant de toucher au niveau.

## 5. Évolution lente : faisable ou figé ?

- Vitesse de 5 cm/an = 50 km/Ma = environ une cellule L7 par million d'années.
- Sur 10 000 ans simulés (exigence du PROJET) : ~0,5 km, moins d'un centième de cellule : **figer**.
- Au-delà de quelques millions d'années : reconstruire la carte géologique (graines déplacées par Euler, frontières
  reclassées, coût ~100 ms à L7 **[estimation]**) toutes les 5 à 10 Ma simulées, et advecter l'élévation par
  lookup semi-lagrangien.
- Problème non résolu à ce stade : création et destruction de croûte (subduction) doit conserver une masse ; à traiter
  avec un bilan explicite, pas au MVP.

**Recommandation** : MVP figé (E1 à E5). Évolution tectonique = nouvel épique post-E5, avec ses propres tests de masse.

---

## Règles pour Tellus

Pipeline de genèse, déterministe, une seule passe. Chaque étape est une fonction pure (graine, grille) → champ.

```
G0 grille(L)            : icosphère L ; sommets, cellules (duaux), voisins CSR, aires exactes ; lloyd(3 itérations)
G1 plaques(graine, N)   : rng = shared::rng(graine) ; N graines ; Dijkstra multi-sources (distance pondérée par bruit)
                          plaque_type ∈ {océan, continent}, au moins un de chaque
G2 mouvement            : u_p aléatoire ; ω_p ∈ [1, 15] cm/an / R ; v_p(x) = R·ω_p·(u_p × x̂)
G3 frontières           : pour chaque arête (a,b) de plaques différentes : c, s, type ∈ {conv, div, transf}
G4 profils              : pour chaque type de frontière : distance multi-sources ; h_tect = Σ A·exp(−d²/2σ²)
G5 âge & profondeur     : âge = d_dorsale / v_demi ; océan : h = −GDH1(âge) ; continent : h = (T−35 km)·(ρ_m−ρ_c)/ρ_m
G6 relief               : h += α · fBm(x̂) · masque_continent ; orogènes += ridged(x̂)
                          érosion_stream_power(100 itérations) ; érosion_thermique(10 itérations) ; isostasie
G7 niveau de la mer     : s = quantile_aire(h, 0.708) ; h −= s ; calibrer α sur altitude moyenne des terres (840 m)
G8 contrôles            : assert finitude ; assert tests de la section suivante ; sinon erreur + log, pas de planète partielle
```

Règles transverses :
- Tout aléa passe par `shared::rng` (graine + identifiant de l'étape) ; aucun `thread_rng`.
- Une étape ne lit que les champs produits par les étapes précédentes (`genesis/` ne touche ni `climate/` ni `life/`).
- Chaque fonction est ≤ 35 lignes ; les étapes G1 à G7 sont des fichiers séparés dans `simulation/src/genesis/`.
- Les constantes physiques (ρ, T_ref, GDH1) vivent dans `shared/` (unités).

---

## Tests de validation

Chaque ligne = un `#[test]` cargo. Seuils chiffrés ; si un seuil est trop strict, il se corrige ici, pas dans le test.

**Grille (G0)**
- `grid_cell_count` : N(L) = 10·4^L + 2 pour L ∈ {6, 7, 8} (40 962 / 163 842 / 655 362).
- `grid_valence` : exactement 12 cellules à 5 voisins ; toutes les autres à 6 (L ≥ 2).
- `grid_area_sum` : Σ A_i = 4πR² à 10⁻⁹ relatif.
- `grid_area_uniformity` : max A / min A < 1,25 après lloyd (L7).

**Déterminisme (transverse)**
- `genesis_deterministic` : même graine → hash FNV-64 identique sur h, deux exécutions.
- `genesis_seed_diversity` : 10 graines → 10 hashs distincts.
- `genesis_finite` : aucun NaN ni infini dans les champs finaux.

**Plaques et mouvement (G1, G2)**
- `plate_count` : N_p ∈ [8, 14].
- `plate_connected` : chaque plaque est connexe (BFS sur voisins).
- `plate_types` : au moins une plaque océanique et une continentale.
- `velocity_tangent` : |v_p(x) · x̂| < 10⁻⁹ · |v_p(x)| pour toutes les cellules.
- `velocity_range` : |v_p| ∈ [1, 15] cm/an pour toutes les plaques.

**Frontières (G3, G4)**
- `boundary_convergent` : deux plaques qui se rapprochent → classées convergentes ; signe inversé → divergentes ;
  mouvement parallèle → transformantes (cas synthétiques, 3 tests).
- `trench_depth` : fosse océan–continent entre −6 000 et −8 000 m avant isostasie.
- `orogen_height` : chaîne continent–continent entre +3 000 et +5 000 m.

**Âge et profondeur (G5)**
- `gdh1_continuity` : |d(20⁻) − d(20⁺)| < 5 m.
- `gdh1_values` : d(0) = 2600 m ; d(100) ∈ [5 450, 5 550] m.
- `oceanic_depth_mean` : profondeur moyenne océanique ∈ [3 400, 4 000] m (cible 3 700 ± 300 m).
- `airy_isostasy` : T = 45 km → h = 1 818 ± 1 m.

**Niveau de la mer et hypsométrie (G7)**
- `sea_fraction` : |f_eau − 0,708| ≤ 0,005.
- `land_fraction` : f_terres ∈ [0,262 ; 0,322] (cible 0,292 ± 3 points, tolérance demandée par Chris).
- `land_mean_elevation` : altitude moyenne des terres ∈ [690, 990] m (cible 840 ± 150 m).
- `ocean_mean_depth` : profondeur moyenne des océans ∈ [3 400, 4 000] m.
- `hypsometry_bimodal` : histogramme pondéré par aire (pas de 100 m) → deux maxima locaux ; le premier dans
  [−200, +400] m, le second dans [−5 000, −3 500] m ; densité au creux < 0,5 × min(pics).

**Érosion (G6)**
- `erosion_mass_balance` : |Σ érodé − Σ déposé| / Σ érodé < 10⁻⁶.
- `erosion_monotone` : après érosion, aucune cellule continentale n'est plus haute qu'avant (sans soulèvement).

**Figement et performance**
- `tectonics_frozen_mvp` : après 10 000 ans simulés, les plaques et l'élévation sont inchangées (hash égal).
- `genesis_budget` : `cargo bench genesis` L7 < 5 s sur 6 cœurs.
- `tick_budget` : `cargo bench tick` L7 < 8 ms médiane, 4 workers (E3.S5).

---

## Sources

- Cortial et al. 2019, CGF 38(2) — https://diglib.eg.org/handle/10.1111/cgf13614 — DOI 10.1111/cgf.13614 [vérifié]
- Cortial et al. 2020, « Real-Time Hyper-Amplification of Planets », The Visual Computer [vérifié, titre]
- Górski et al. 2005, « HEALPix », astro-ph/0409513 — https://arxiv.org/abs/astro-ph/0409513 [vérifié]
- González 2010, « Measurement of areas on a sphere using Fibonacci and latitude–longitude lattices », Math. Geosci., arXiv:0912.4540 — https://arxiv.org/pdf/0912.4540 [vérifié]
- Stein & Stein 1992, Nature 359, 123–129 [vérifié, citation] ; formules GDH1 : https://docs.generic-mapping-tools.org/6.1/gallery/ex49.html [vérifié]
- Davis & Lister 1974, EPSL 21 ; Parsons & Sclater 1977, JGR 82 [vérifié via https://en.wikipedia.org/wiki/Seafloor_depth_versus_age]
- Braun & Willett 2013, Geomorphology 180–181 — https://landlab.readthedocs.io/en/latest/reference/components/stream_power.html [vérifié]
- Beneš & Forsbach 2002, « Visual simulation of hydraulic erosion » — https://www.cs.purdue.edu/cgvlab/www/resources/papers/Benes-2002-Visual_simulation_of_hydraulic_erosion.pdf [vérifié]
- Ronchi, Iacono, Paolucci 1996, « The cubed sphere », J. Comput. Phys. 124 — [mémoire, non vérifié] ; citation indirecte via https://arxiv.org/pdf/2403.06844 [vérifié]
- Charette & Smith 2010, Oceanography — profondeur moyenne 3 682,2 m [vérifié, résumé de recherche]
- Britannica, hypsométrie — https://www.britannica.com/print/article/280163 [vérifié]
- Goldberg polyhedron, comptage 10·T + 2 — https://en.wikipedia.org/wiki/Goldberg_polyhedron [vérifié]
- math33, gamedev.net — https://gamedev.net/blogs/entry/2262882-how-i-try-to-create-entire-scientifically-realistic-worlds-with-math33 [vérifié résumé]

Points non trouvés ou non vérifiés : corps du papier Cortial (PDF bloqué), PlaTec, Experilous, Beyer 2015 (introuvable),
numéro exact de la chaîne GDH1 au-delà de 20 Ma (formule confirmée par GMT, source primaire non lue).
