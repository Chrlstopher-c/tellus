# Planètes du système solaire : données de presets Tellus

Données : `simulation/data/planets.json` (11 corps : 8 planètes, Lune, Titan, Europe).
Sources principales : NASA NSSDC Planetary Fact Sheets, NASA Science (pages Facts), JPL Solar System Dynamics
(paramètres physiques des satellites), un article arXiv sur la température de surface d'Europe.

## 1. Sources

| Code | Source |
|---|---|
| NSSDC-P | https://nssdc.gsfc.nasa.gov/planetary/factsheet/ (`mercuryfact`, `venusfact`, `earthfact`, `marsfact`, `jupiterfact`, `saturnfact`, `uranusfact`, `neptunefact`, `moonfact`, `joviansatfact`) |
| NASA-M | https://science.nasa.gov/mercury/facts/ |
| NASA-V | https://science.nasa.gov/venus/facts/ |
| NASA-Ma | https://science.nasa.gov/mars/facts/ |
| NASA-L | https://science.nasa.gov/moon/facts/ |
| NASA-T | https://science.nasa.gov/saturn/moons/titan/facts/ |
| JPL-S | https://ssd.jpl.nasa.gov/sats/phys_par/ (GM et rayon moyen des satellites, éphémérides JUP365 et SAT441) |
| ARX-E | https://arxiv.org/pdf/1608.07372 (température de surface d'Europe, modèle) |
| JPL-E | https://www.jpl.nasa.gov/edu/teach/activity/frozen-formula-a-pi-in-the-sky-math-challenge (épaisseurs de glace/océan, fourchettes d'hypothèses) |
| PDS | https://pds-rings.seti.org/press_releases/pages/PIA19xxx/PIA19055.html (Titan, pas de champ intrinsèque) |

Les pages NASA Europa et Titan du domaine `solarsystem.nasa.gov` redirigent vers `science.nasa.gov/...`. Europa n'a pas pu être lue
sur la page NASA (404) ; ses valeurs viennent de JPL-S, NSSDC-P et ARX-E.

## 2. Tableau récapitulatif

Rayon = rayon moyen volumétrique. Pression = pression de surface (bar). Température : moyenne / min / max (K).
Flux = flux solaire reçu à la distance du corps (W/m²), avec 1361 W/m² à 1 UA. Pour Titan et Europe, flux du corps
parent (Saturne, Jupiter) à sa distance du Soleil.

| Corps | Type | Masse (kg) | Rayon (km) | g (m/s²) | a (UA) | e | Obliq. (°) | Rot. sid. (h) | Jour solaire (h) | P orb. (j) | A Bond | Flux (W/m²) | P surf (bar) | T moy / min / max (K) | Surface solide |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mercure | tellurique | 3.3010e23 | 2439.7 | 3.70 | 0.38710 | 0.2056 | 0.034 | 1407.6 | 4222.6 | 87.969 | 0.068 | 9082.7 | < 5e-15 | 440 / 93 / 725 | oui |
| Vénus | tellurique | 4.8673e24 | 6051.8 | 8.87 | 0.72333 | 0.0068 | 177.36 | -5832.6 | 2802.0 | 224.701 | 0.77 | 2601.3 | 92 | 737 / n.d. / n.d. | oui |
| Terre | tellurique | 5.9722e24 | 6371.0 | 9.82 | 1.00000 | 0.0167 | 23.44 | 23.9345 | 24.0 | 365.256 | 0.294 | 1361.0 | 1.014 | 288 / 283 / 293 * | oui |
| Mars | tellurique | 6.4169e23 | 3389.5 | 3.73 | 1.52366 | 0.0935 | 25.19 | 24.6229 | 24.6597 | 686.980 | 0.250 | 586.2 | 0.00636 | 214 / 120 / 293 | oui |
| Jupiter | gazeuse | 1.89813e27 | 69911 | 25.92 (1 bar) | 5.20336 | 0.0487 | 3.13 | 9.9250 | 9.9259 | 4332.589 | 0.343 | 50.26 | (> 1000 bar) | T(1 bar) = 165 | non |
| Saturne | gazeuse | 5.6832e26 | 58232 | 11.19 (1 bar) | 9.53707 | 0.0520 | 26.73 | 10.656 | 10.656 | 10755.699 | 0.342 | 14.82 | (1 bar) | T(1 bar) = 134 | non |
| Uranus | gazeuse | 8.6811e25 | 25362 | 9.01 (1 bar) | 19.19126 | 0.0469 | 97.77 | -17.24 ** | 17.24 | 30685.400 | 0.300 | 3.69 | (1 bar) | T(1 bar) = 76 | non |
| Neptune | gazeuse | 1.02409e26 | 24622 | 11.27 (1 bar) | 30.06896 | 0.0097 | 28.32 | 16.11 ** | 16.11 | 60189.018 | 0.290 | 1.508 | (1 bar) | T(1 bar) = 72 | non |
| Lune | satellite rocheux (Terre) | 7.346e22 | 1737.4 | 1.62 | 0.0025696 | 0.0549 | 6.68 | 655.72 | 708.7 *** | 27.3217 | 0.11 | 1361.0 | 3e-15 | n.d. / 95 / 390 | oui |
| Titan | satellite glacé (Saturne) | 1.34518e23 | 2574.76 | 1.354 | 0.0081677 | n.d. ‡ | n.d. ‡ | 382.7 ‡ | 382.7 ‡ | 15.945 | 0.22 ‡ | 14.82 | 1.6 | 94 / n.d. / n.d. | oui |
| Europe | satellite glacé (Jupiter) | 4.79857e22 | 1560.80 | 1.315 | 0.0044860 | 0.009 | n.d. | 85.228 | 85.228 | 3.551181 | n.d. (géom. 0.68) | 50.26 | exosphère | ~100 (modèle) / 52 / 110 | oui (glace) |

Notes du tableau :
- \* Terre : min/max = plage diurne du fact sheet, pas les extrêmes globaux.
- \*\* Uranus et Neptune : périodes de rotation issues des coordonnées magnétiques (Voyager 2) ; signe négatif pour Uranus.
- \*\*\* Lune : jour synodique 29.53 j.
- ‡ Titan : valeurs non vérifiées sur une source primaire (voir section 7). Masse et gravité dérivées de GM (JPL-S) ; rotation synchrone.
- n.d. : non publié dans les sources lues. Vénus : variation diurne ~0 K, donc pas de min/max utile.

## 3. Champ magnétique, eau, géologie

| Corps | Champ magnétique | Eau | Tectonique | Volcanisme |
|---|---|---|---|---|
| Mercure | oui, dipôle décalé (0.002 G·Rh³, décalage 0.17 Rh), ~1 % de la Terre en surface | glace probable dans les cratères polaires ombragés | contraction (falaises ~1 mile) | non documenté dans la source |
| Vénus | non (champ induit uniquement) | aucune en surface ; ancien océan possible | subduction possible, non confirmée | milliers de volcans, dômes |
| Terre | oui, 0.24–0.66 G en surface | océans, glaces polaires | plaques (connaissance générale) | actif (connaissance générale) |
| Mars | non aujourd'hui (magnétisation crustale fossile) | glace près de la surface aux pôles ; saumures saisonnières ; anciens lacs/rivières | mouvement crustal, peu détaillé | Olympus Mons (~25 km) |
| Jupiter | oui, 4–13 G | aucune en surface | aucune | aucun |
| Saturne | oui, 0.18–0.84 G (champ non dipolaire) | aucune en surface | aucune | aucun |
| Uranus | oui, 0.1–1 G, axe incliné de 58.6° | aucune en surface | aucune | aucun |
| Neptune | oui, 0.1–0.9 G, axe incliné de 46.9° | aucune en surface | aucune | aucun |
| Lune | non aujourd'hui (dynamo ancienne) | glace dans cratères polaires ombragés ; OH/H2O détectés | aucune active | éteint, maria 4.2 à 1.2 Ga |
| Titan | non (champ induit par Saturne) | lacs de méthane/éthane ; océan sous-glaciaire 55–80 km (NASA) | non documentée | cryovolcanisme suspecté, non confirmé |
| Europe | non (champ induit par Jupiter, non sourcé ici) | océan liquide sous glace ; glace 3–30 km, océan 3.5–100 km (fourchettes JPL-E) | fractures, terrains chaotiques (chauffage de marée, non sourcé ici) | pas de volcanisme silicaté ; cryo non confirmé |

## 4. Ce que Tellus doit dériver de ces paramètres

### 4.1 Température d'équilibre
T_eq = (S(1 − A) / (4σ))^0,25, avec σ = 5.670e-8 W/(m²·K⁴), S = flux solaire local, A = albédo de Bond.

Pour une planète tournant lentement, la répartition nuit/jour se moyenne ; c'est la formule globale à utiliser.

### 4.2 Effet de serre
Effet de serre = T_surface − T_eq. Pour les planètes telluriques, il s'exprime avec la pression et la composition
(CO2, H2O, CH4). Vénus : 737 − 227 ≈ 510 K. Terre : 288 − 255 ≈ 33 K. Mars : 214 − 210 ≈ 4 K.

Les géantes n'ont pas de surface. Leur température à 1 bar (165, 134, 76, 72 K) est supérieure à T_eq à cause de la chaleur
interne (Jupiter : 165 K contre T_eq 110 K), pas à cause d'un effet de serre de surface. Tellus ne doit pas utiliser la
même formule pour elles sans un terme de chaleur interne.

### 4.3 Durée du jour solaire
Pour une rotation prograde (sens de l'orbite) : 1/P_solaire = 1/P_sidéral − 1/P_orbital.
Pour une rotation rétrograde (Vénus, Uranus) : 1/P_solaire = 1/|P_sidéral| + 1/P_orbital.

Vérifications : Terre 23.9345 h et 365.256 j donnent 24.0 h. Vénus donne 2802 h, Mercure donne 4224 h (fact sheet : 4222.6 h).
Pour les satellites en rotation synchrone (Titan, Europe), jour solaire = période orbitale.

### 4.4 Flux et gravité
- Flux à la distance d : S = 1361 / d² (d en UA). Vérification Mars : 1361 / 1.5237² ≈ 586.4 (fact sheet : 586.2).
- Gravité surface : g = GM / r². Titan : 8978.137 / 2574.76² ≈ 1.354 m/s² ; Europe : 3202.712 / 1560.8² ≈ 1.315 m/s².

## 5. Valeurs de validation attendues

| Corps | T_eq calculée (A = Bond) | T_surface attendue | Effet de serre attendu |
|---|---|---|---|
| Terre | 255.1 K (fact sheet bb : 254.0 K) | 288 K | ~33 K |
| Vénus | 226.6 K (bb : 226.6) | 737 K | ~510 K |
| Mars | 209.8 K (bb : 209.8) | 214 K | ~4 K |
| Mercure | 439.5 K (bb : 439.6) | 440 K | ~0 K |
| Lune | 270.3 K (bb : 270.4) | non publiée | ~0 (pas d'atmosphère) |
| Titan | 84.5 K (A = 0.22, non vérifié) | 94 K | ~9.5 K |
| Jupiter | 109.8 K | T(1 bar) = 165 K | chaleur interne, pas de serre |

Tests de cohérence à coder : jour solaire Terre = 24.0 h ± 0.01 ; T_eq Terre = 255 ± 1 K ; g Terre = 9.82 ; Lune g = 1.62.

## 6. Géantes : traitement dans Tellus

- `surface_solid: false` pour Jupiter, Saturne, Uranus, Neptune. Pas de surface physique dans `planets.json`.
- Non jouable en surface. Surface fictive définie à 1 bar : température, gravité et composition à 1 bar.
- Saturne : le champ n'est pas dipolaire (quadrupôle et octupôle significatifs). Un dipôle décalé reste une approximation.
- Uranus : axe à 97.8° : saisons extrêmes à traiter dans le modèle d'insolation.

## 7. Incertitudes et points à vérifier

- Titan : excentricité, obliquité, albédo de Bond 0.22 et rotation synchrone ne sont pas lus sur une source primaire.
  L'albédo 0.22 est une valeur courante de la littérature, à confirmer. Le demi-grand axe 1 221 870 km est déduit du « 1.2 million km » de NASA.
- Europe : température 100 K (moyenne), 52 K (pôles) et 110 K (équateur) viennent d'un modèle (ARX-E), pas d'une mesure.
  Pas d'albédo de Bond, pas d'obliquité, pas de pression de surface sourcés. Glace 3–30 km et océan 3.5–100 km sont des fourchettes d'hypothèses.
- Mercure : température max 725 K (fact sheet, face éclairée) contre 430 °C = 703 K (NASA-M). Écart de ~20 K, gardé le fact sheet.
- Terre : tectonique et volcanisme ne sont pas sourcés dans cette recherche.
- Lune : température moyenne absente. Le corps noir (270.4 K) n'est pas une moyenne de surface.
- Saturne : demi-grand axe J2000 (9.537 UA) retenu, table d'orbite donne 9.573 UA.
- Uranus, Neptune : périodes de rotation issues des coordonnées magnétiques, donc incertaines de quelques heures.

Aucune donnée sans source n'a été inventée. Les champs `data_flags` de `planets.json` répètent ces points par corps.
