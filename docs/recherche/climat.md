# Tellus — Climat simplifié calculable par cellule et par tick

Recherche documentaire, 2026-11-01. Aucun code écrit. Modèle : claude-haiku-5-5.

Légende : **[V]** page lue en entier · **[S]** source secondaire ou résumé (à confirmer sur l'article) ·
**[M]** connaissance de l'agent, non revérifiée ici · « calc. » = dérivé des formules de ce document (à retester dans `cargo test`).

---

## Recommandation

1. **Cœur** : EBM 2D Budyko–Sellers sur la grille géodésique. Chaque tick ne résout que T(cellule) et son bilan d'énergie.
2. **Insolation** : moyenne journalière (Berger 1978), orbite de Kepler une fois par jour de l'année, table latitude × mois.
3. **Effet de serre** : Terre `OLR = A + B·T − ΔF`, `ΔF = 5,35·ln(C/C0)` ; autres planètes `OLR = σT⁴/(1 + 3τ/4)`.
4. **Glace** : albédo à seuil (T_f = −10 °C, rampe de 10 K). La boule de neige est possible et hystérétique : c'est attendu.
5. **Vents, humidité, précipitations, courants** : diagnostics recalculés par mois (cache), jamais à chaque tick.
6. **Pas de temps** : tick climatique = 1 mois, diffusion implicite ; saison puis an à grande vitesse (§8).
7. **Pas de GCM** (PlaSim, ExoPlaSim) dans le moteur : trop lourds. Références hors ligne pour valider les chiffres.
8. **Calibration Terre** : A = 210 W/m², B = 2,0 W/m²/K, D = 0,555 (climlab). Les valeurs satellite (A = 203,3, B = 2,09)
   donnent T̄ ≈ 290 K, 2 K trop chaud.
9. **Validation** : T̄ ≈ 288 K, gradient équateur–pôle, P̄ ≈ 1000 mm/an, flux latent ≈ 80 W/m², Köppen en gros (§Tests).
10. **Choix non sourcés** (Hadley, orographie, albédos hors glace, échelle des cellules avec Ω) : à calibrer, marqués dans le code.

---

## 1. Insolation, déclinaison, obliquité, excentricité

**Équations** (Berger 1978 ; forme reprise par climlab et les codes paléoclimat) [S]

- Déclinaison : `sin δ = sin ε · sin λ` (λ = longitude vraie du Soleil depuis l'équinoxe de printemps).
- Orbite : `M = n(t − t_p)`, `n = 2π/T_an` ; Kepler `E − e·sin E = M` (Newton, 3 à 5 itérations) ; `ν`, puis `λ = ν + ϖ`.
- Distance : `(a/r)² = (1 + e·cos ν)² / (1 − e²)²`.
- Insolation journalière moyenne au TOA :

  `Q̄(φ, λ) = (S0/π) · (a/r)² · [H0·sin φ·sin δ + cos φ·cos δ·sin H0]`, `H0 = arccos(−tan φ·tan δ)`
  borné : `tan φ·tan δ ≥ 1` → `H0 = π` (jour polaire) ; `≤ −1` → `H0 = 0` (nuit polaire).

**Constantes**

| Grandeur | Valeur | Source |
|---|---|---|
| S0 Terre | 1361,0 W/m² (climlab : 1365,2) | [V] NASA Terre |
| ε, e, Terre | 23,44° ; 0,0167 | [V] NASA Terre |
| ε Mars ; e Mars | 25,19° ; 0,0935 | [V] NASA Mars |
| ε Vénus | 177,36° (rotation rétrograde) | [V] NASA Vénus |
| Plages Milankovitch | e 0–0,06 ; ε 22,1–24,5° ; précession ≈ 26 kyr | [M] Laskar 2004 |

Laskar 2004 (solution numérique) est la référence d'orbite. Un képlérien suffit au MVP.

**Contrôles** (calc.) : équateur à l'équinoxe `S0/π = 433 W/m²` ; pôle au solstice `S0·sin δ = 542 W/m²` ;
moyenne annuelle TOA `S0/4 = 340 W/m²` ; périhélie/aphélie `((1+e)/(1−e))² = 1,069`.

**Saisons et jour/nuit** : `Q̄` est la moyenne sur une rotation, le cycle diurne n'est pas résolu (tick ≥ 1 jour).
Table `Q[bin 1°][mois]` (181 × 12), recalculée seulement si S0, e, ε, ϖ ou la durée d'année changent.

---

## 2. Bilan énergétique (EBM 2D Budyko–Sellers)

**Équation** [V climlab, page et code `ebm.py`]

`C(φ)·∂T/∂t = Q̄·(1 − α) − OLR − ΔF(C) + (D/cos φ)·∂/∂φ(cos φ·∂T/∂φ)`

`φ` en radians ; `D` en W/m²/K avec `∇²` en rad⁻². Alternative Budyko-transport : `H = −b·(T − T̄)`, `b = 3,81` [V].

| Paramètre | Valeur retenue | Origine |
|---|---|---|
| A | 210 W/m² | [V] climlab `EBM` (calé sur 288 K) |
| B | 2,0 W/m²/K | [V] climlab `EBM` (inclut vapeur et lapse rate) |
| D | 0,555 W/m²/K | [V] climlab `EBM` |
| A, B satellite | 203,3 ; 2,09 | [S] UW handout — écarté, trop chaud |
| A, B, Q, C autres | 202 ; 1,90 ; 343 ; 3,04 | [S] Cornell / McGehee |

**Contrôle d'équilibre** (calc.) : `T̄ = (S0/4·(1 − α) − A)/B`, S0 = 1361, α = 0,30 →
A = 210, B = 2,0 : **287,2 K** (retenu) ; A = 203,3, B = 2,09 : 289,9 K (écarté).

**Capacités thermiques** (calc., `C = ρ·c·H`, eau : ρc = 4,18·10⁶ J/m³/K [S climlab])

| Surface | H | C (J/m²/K) | Remarque |
|---|---|---|---|
| Océan de test rapide | 10 m | 4,18·10⁷ | [V] climlab `water_depth` |
| Océan mélangé de référence | 70 m | 2,93·10⁸ | [S] moyenne globale, ≈ 30 × atmosphère |
| Colonne d'air | — | ≈ 1,0·10⁷ | [S] climlab |
| Sol / désert (2 m) | 2 m | ≈ 4·10⁶ | [M] |

Temps caractéristiques τ = C/B : 10 m → 0,66 an ; 70 m → 4,4 ans (calc.).
D unique pour terre et océan, comme climlab ; une diffusion réduite sur continent est à calibrer.

---

## 3. Effet de serre et autres atmosphères

**CO2** [S : IPCC TAR cite Myhre et al. 1998 ; Science of Doom] : `ΔF = 5,35·ln(C/C0)` W/m².
Doublement : 3,71 W/m² (calc.). C0 : 275, 278 ou 280 ppm selon les sources ; retenu **280 ppm**.
Sensibilité sans rétroaction (calc.) : `ΔF/B` = 1,77 K (B = 2,09) ou 1,85 K (B = 2,0).
La vapeur d'eau n'est pas explicite : elle est dans B.

**Autres planètes** — atmosphère grise [M] : `T_e = [S(1−α)/4σ]^(1/4)`, `T_s = T_e·(1 + 3τ/4)^(1/4)`,
`τ = (4/3)·[(T_s/T_e)⁴ − 1]`.

| Planète | S (W/m²) | α | T_e calc. (K) | T_s NASA (K) | τ déduit |
|---|---|---|---|---|---|
| Terre | 1361 | 0,294 | 255,6 | 288 | 0,8 |
| Vénus | 2601 | 0,77 | 226 | 737 | ≈ 150 |
| Mars | 586 | 0,25 | 210 | 214 | 0,1 |

S, α, T_s : [V] NASA. τ(P_s, composition) : loi non linéaire, γ ≈ 1,15 entre Terre et Vénus (calc.), à calibrer [M].

**Décision** : Terre en A + B·T − ΔF (calibré, reproduit 288 K) ; autres planètes en OLR grise, sans rétroaction
vapeur. Vénus ne sera pas reproduite (emballement hors domaine) : le test enregistre 737 K comme écart attendu.

---

## 4. Albédo, glace, boule de neige

`α = α_glace` si T < T_f − 5 K ; `α = a0 + a2·P2(sin φ)` si T > T_f + 5 K ; rampe linéaire entre les deux [décision].

| Paramètre | Valeur | Source |
|---|---|---|
| a0 ; a2 | 0,30 ; 0,078 | [V] climlab `EBM` |
| α glace | 0,62 (variante sans glace 0,32) | [V] climlab ; [S] McGehee & Widiasih |
| T_f | −10 °C (variante climlab commentée : 0 °C) | [V] climlab |

Autres albédos (océan ≈ 0,06–0,10 ; forêt ≈ 0,10–0,15 ; désert ≈ 0,30–0,40 ; neige fraîche ≈ 0,8) : [M], à remplacer
par une table de référence. Glace de mer : fraction `f = rampe(T)`, diagnostic, sans dynamique (MVP).

**Boule de neige** : la rétroaction glace-albédo donne deux équilibres pour un même forçage [M, Budyko 1969, Sellers 1969].
Seuil à mesurer dans le modèle (ordre de 0,9 à 0,95 S0 [M]).

---

## 5. Circulation atmosphérique

**Fondement** : Held & Hou (1980), cellule de Hadley axisymétrique stationnaire. L'air en altitude conserve
`M = (u + Ω·a·cos φ)·a·cos φ` [S, notes NOAA et Harvard]. Parti du repos à l'équateur :

`u_haut(φ) = Ω·a·sin²φ / cos φ` (calc., forme standard)

Terre : Ωa ≈ 464 m/s ; à 30°, u ≈ 134 m/s. **Surestimé** d'un facteur 3 à 4 par rapport au jet observé (≈ 30 m/s [M]),
et la cellule qui en sort est trop étroite et trop faible [S, UCI]. On ne retient que la **forme** `sin²φ / cos φ`,
la magnitude est plafonnée par friction.

**Paramétrage** : bord de cellule φ_H ≈ 30° sur Terre [M]. Dépendance à Ω : les sources se contredisent (Ω^(−1/2) chez
Held–Hou, « largeur ∝ 1/Ω » chez UCI [S]). Décision : `φ_H = clamp(30°·(Ω/Ω_T)^(−k), 15°, 45°)`, `k` à calibrer.
Vents de surface : alizés (Est sous 30°), westerlies (30–60°), Est polaires (> 60°) [M] ; déviation Ekman ≈ 20–45° [M].
Coriolis `f = 2Ω·sin φ`, nul à l'équateur : régularisation `|f| ≥ f_min` [décision].
Sortie : `u, v` par cellule et par mois, issus d'une fonction de courant ψ(φ) à trois cellules, précalculée par planète.

---

## 6. Humidité et précipitations

**Saturation** — Bolton (1980) [S, Vömel ; Pyrcel] : `e_s = 6,112·exp(17,67·T/(T + 243,5))` hPa, T en °C,
précision 0,1 % sur −30 à 35 °C. Valeurs (calc.) : 6,11 hPa à 0 °C ; 17,04 hPa à 15 °C ; 42,4 hPa à 30 °C.
`q_sat = 0,622·e_s/p` [M] : 10,5 g/kg à 15 °C, 1013 hPa.

**Clausius–Clapeyron** (calc.) : `d ln e_s/dT = 17,67·243,5/(T + 243,5)²` = **7,3 %/K à 0 °C, 6,4 %/K à 15 °C, 5,8 %/K à 30 °C**.

**Évaporation** : océan `E = ρ_a·C_E·|U|·(q_sat(T_s) − q_a)`, `C_E ≈ 1,2·10⁻³` [M] ; sur sol, `E × β` (humidité du sol).
Contrôle (calc.) : 1000 mm/an = `1 m × 1000 kg/m³ × 2,5·10⁶ J/kg / 3,156·10⁷ s` = **79 W/m²**, cohérent avec ≈ 80 W/m² [M].

**Précipitations** : convergence de vapeur `P − E = −∇·(q·U)` (mensuel, semi-lagrangien). Condensation si `q > q_sat` :
`P = (q − q_sat)/τ_p`, `τ_p ≈ 1–2 h` [M].
**ITCZ** : émerge de la convergence des alizés, suit la déclinaison avec un retard [M]. Aucun paramètre ITCZ à poser.

**Orographie** — Smith & Barstad (2004), modèle linéaire en Fourier [S, équations non relues] : τ_c, τ_f ≈ 1000 s,
H_w ≈ 2,5 km [M]. Implémentation : **noyau de convolution précalculé** par direction de vent saisonnière (≈ 8 × 12),
relief fixe. Partie positive du noyau = pluie au vent, lobe négatif = ombre pluviométrique. Recalcul si le relief change.

---

## 7. Océan : courants, transport, glace

- Couche mélangée : 70 m (référence), 10 m (tests rapides).
- Transport : (a) diffusion D, MVP ; (b) Budyko-transport `H = −b·(T − T̄)`, b = 3,81 [V]. (b) est peu coûteux mais ne redistribue pas zonalement.
- Gyres : fonction de courant ψ_gyre fixée par planète, pondérée par le rotationnel du vent (Sverdrup [M]). Module optionnel, hors MVP.
- Contrôle : transport méridien de pointe ≈ 5 PW par hémisphère vers 35–40° [M]. Valeur de contrôle, pas une entrée.
- Glace de mer : fraction f (§4), albédo recalculé. Eau de mer : gel vers −1,8 °C [M].

---

## 8. Pas de temps, découplage, stabilité

**Grille** : géodésique icosaédrique, `N = 10·4ⁿ + 2` ; n = 6 → **40 962** cellules (h ≈ 0,0175 rad), n = 7 → **163 842** (h ≈ 0,0088) (calc.).

**Deux familles de pas** :

| Famille | Pas | Contenu | Coût |
|---|---|---|---|
| Lente | 1 mois (défaut), saison, an | T, glace, albédo | O(N) par tick, diffusion implicite |
| Diagnostic | cache mensuel, recalcul tous les ~10 ans simulés ou si |ΔT| global > seuil | vent, P, E, Köppen | hors chemin critique |

**Coût** : `ticks/s = V/60 × pas_par_an` (V en années simulées par minute). Budget : 8 ms par tick.

| Pas | pas/an | Vitesse max sous 8 ms | Ticks/s à 10 000 y/min | Budget à 10 000 y/min |
|---|---|---|---|---|
| Mois | 12 | ≈ 600 y/min | 2000 | 0,5 ms (impossible à 160 k) |
| Saison | 4 | ≈ 1 800 y/min | 667 | 1,5 ms (difficile) |
| An | 1 | ≈ 7 500 y/min | 167 | 6 ms (faisable) |

Règle : mois jusqu'à 600 y/min, saison jusqu'à 1 800, an au-delà.

**Diffusion explicite** (calc., `C·h²/(4D)`, 2D, à valider numériquement) :

| Cellules | C = 4,18·10⁷ (10 m) | C = 2,93·10⁸ (70 m) |
|---|---|---|
| 40 962 | 1,6 h | 11 h |
| 163 842 | 24 min | 2,8 h |

→ **implicite obligatoire** à 1 mois sur 160 k cellules : Crank–Nicolson ou Euler implicite, résolu par Jacobi ou CG
(5 à 10 itérations, matrice creuse à 6 voisins).

**Advection de vapeur** : CFL explicite = 56 km / 20 m/s ≈ 47 min (160 k). Semi-lagrangienne (inconditionnellement stable).

**Oscillations** : une coupure d'albédo sèche fait osciller la glace entre deux ticks. Rampe de 10 K et relaxation sur T.

**Précision** : f32 sur GPU, bilans globaux accumulés en f64 côté CPU.

---

## 9. Validation : Köppen et chiffres attendus

| Grandeur | Cible | Tolérance |
|---|---|---|
| T̄ globale | 288 K | ± 1,5 K |
| Gradient équateur–pôle (annuel) | ≈ 40 K | ± 5 K ; **à vérifier** selon la latitude polaire de référence |
| Précipitations moyennes | ≈ 1000 mm/an | ± 100 mm/an |
| Flux latent global | ≈ 80 W/m² | ± 5 W/m² |
| Part BWh (désert chaud) des terres | 14,2 % | [S] Peel 2007 |
| Part Aw (savane tropicale) des terres | 11,5 % | [S] Peel 2007 |

**Köppen** [S Kottek 2006, Peel 2007 ; règles de groupe en [M]], sur moyennes mensuelles :
A : T_min ≥ 18 °C ; B : P_ann < 20·T̄ + {280 si ≥ 70 % des pluies en été ; 0 si en hiver ; 140 sinon} [M] ;
C : −3 < T_min ≤ 18 °C et T_max ≥ 10 °C ; D : T_min ≤ −3 °C et T_max ≥ 10 °C ; E : T_max < 10 °C.
Critère : accord de groupe majeur sur **≥ 55 %** des cellules terrestres (un EBM ne reproduit pas les sous-types).

---

## Règles pour Tellus (pseudo-code, ordre par tick)

```
// --- Initialisation, une fois par planète ---
charger S0, ε, e, ϖ, T_an, Ω, a, g, P_s, composition, C_cell (ou H_mix), A, B (Terre) ou τ (grise)
grille géodésique N = 10·4^n + 2 ; par cellule : φ, sinφ, aire, type de surface, relief, C_cell
précalcul : Q[bin 1°][mois] ; matrice de diffusion creuse ; ψ(φ) ; noyau orographique par direction

// --- Tick lent (Δt = 1 mois ; saison / an si V > 600 / 1800 y/min) ---
1. orbite : t → M → E (Newton) → ν → λ ; δ(t) ; (a/r)²(t)                 [O(1)]
2. insolation : Q̄_c = interpolation(Q[φ_c][mois])   (à grande vitesse : moyenne sur le pas)
3. forçage :  Terre : OLR_c = A + B·T_c − 5,35·ln(C_CO2/280)
              autre : OLR_c = σ·T_c⁴ / (1 + 3τ/4), τ = f(P_s, composition, C)
4. albédo : α_c = rampe(T_c, T_f = −10 °C, 10 K) entre 0,62 et 0,30 + 0,078·P2(sin φ)
5. diffusion implicite :
     (I − Δt/C·D·L)·T* = T + Δt/C·(Q̄·(1 − α) − OLR)     [L : Laplacien géodésique, 5–10 itér. CG]
   optionnel : H = −b·(T − T̄) à la place de D·L
6. T ← clamp(T*, T_min, T_max) ; log si borne atteinte
7. glace : f_ice_c = rampe(T_c) ; α mis à jour pour le tick suivant
8. [debug] Σ_c C_c·ΔT_c·aire_c − Σ flux·Δt·aire_c ≈ 0 (1e-6 relatif)

// --- Tick diagnostic (cache mensuel, hors chemin critique) ---
9. si (≥ 10 ans simulés depuis le dernier calcul) ou (|ΔT_global| > seuil) :
     vents : u, v depuis ψ(φ) et Coriolis (f_min)
     humidité : q_sat(T) (Bolton) ; E = ρ·C_E·|U|·(q_sat − q)
     précipitations : P = −∇·(q·U) + E (semi-lagrangien) ; + orographie (noyau)
     stockage P[c,m], E[c,m], U[c,m]

// --- À la demande ---
10. Köppen mensuel ; P̄, T̄, ΔT pôle–équateur ; flux latent global ; dérive
```

---

## Tests de validation

Les identifiants `V-xx` se traduisent en `cargo test` (modules `climate::...`). Un test sur une valeur non calculée reste
en attente. Seuils : §1 à §9.

**Insolation** (`climate::insolation`)
- V-01 : Q̄ équateur à l'équinoxe = 433 ± 1 W/m². V-02 : Q̄ pôle au solstice = 542 ± 1 ; pôle en hiver = 0.
- V-03 : moyenne annuelle TOA = 340 ± 1 W/m². V-04 : ratio périhélie/aphélie = 1,069 ± 0,001. V-05 : |δ| ≤ ε + 0,01°.

**Bilan** (`climate::ebm`)
- V-06 : T̄ (A = 210, B = 2, α = 0,30, S0 = 1361) = 287,2 ± 1,5 K.
- V-07 : gradient équateur–pôle annuel = 40 ± 5 K [à confirmer, §9].
- V-08 : 2×CO2 = 3,71 ± 0,01 W/m² ; ΔT sans rétroaction = ΔF/B ± 0,05 K.
- V-09 : |absorption SW − OLR| < 0,5 W/m² à l'équilibre.
- V-10 : stabilité : Δt = 1 mois, N = 163 842 → ΔT par tick < 5 K, pas d'oscillation de signe.

**Planètes** (`climate::planets`)
- V-11 : T_e = 255,6 ± 1 K (Terre), 226 ± 3 K (Vénus), 210 ± 3 K (Mars).
- V-12 : τ déduit = 0,8 ± 0,2 (Terre), 0,1 ± 0,05 (Mars), 150 ± 20 (Vénus).
- V-13 : Mars calculée à 214 K contre 210 K en modèle : écart < 6 K.
- V-14 : Vénus calculée < 300 K contre 737 K NASA : **échec attendu**, documenté.

**Glace** (`climate::ice`)
- V-15 : fraction de glace = 0 au-dessus de T_f + 5 K, = 1 en dessous de T_f − 5 K.
- V-16 : hystérésis : gelé à S = 0,95·S0 reste gelé ; tempéré au même S reste tempéré. Seuil mesuré et consigné.
- V-17 : à S0, l'état gelé fond en moins de 200 ans simulés.

**Hydrologie** (`climate::hydro`)
- V-18 : e_s = 6,11 ± 0,01 (0 °C), 17,04 ± 0,02 (15 °C), 42,4 ± 0,1 hPa (30 °C).
- V-19 : d ln e_s/dT à 15 °C = 6,4 ± 0,2 %/K. V-20 : 1000 mm/an = 79 ± 2 W/m².
- V-21 : |P̄ − Ē|/P̄ < 1 % sur 100 ans simulés.
- V-22 : P̄ = 1000 ± 100 mm/an ; flux latent = 80 ± 5 W/m² ; maximum de P dans ±10° à l'équinoxe ; P(30°) < 0,5·P(0°).
- V-23 : crête de 2000 m : P sous le vent < 0,5·P au vent (qualitatif).

**Circulation** (`climate::wind`)
- V-24 : u_haut(30°) = 134 ± 2 m/s (forme Held–Hou sans friction ; test de forme seulement).
- V-25 : |f| ≥ f_min à l'équateur, f_min consigné.

**Köppen** (`climate::koppen`)
- V-26 : accord de groupe majeur ≥ 55 % des cellules terrestres. V-27 : BWh des terres dans [10 %, 20 %].

**Performance** (`climate::perf`)
- V-28 : tick mensuel à N = 163 842 ≤ 8 ms à 600 y/min. V-29 : tick annuel à 10 000 y/min ≤ 6 ms.

**Déterminisme** (`shared::rng`)
- V-30 : même graine → mêmes T, P, E sur 1000 ans simulés (bit-à-bit CPU ; tolérance 1e-6 sur GPU).

---

## Points non vérifiés et conflits

- Berger 1978 et Laskar 2004 : formules et plages de [S]/[M], articles non relus. Vérifier l'éq. 10 de Berger.
- Smith & Barstad 2004 : équations non relues proprement ; τ_c, τ_f, H_w en [M].
- Peel 2007 : seuils Köppen en [M] ; vérifier le tableau de l'article (supplément libre d'accès).
- Held–Hou : sources en conflit sur la dépendance en Ω. Paramètre `k` à calibrer, pas une loi.
- D de North 1975 : non retrouvé. D = 0,555 vient de climlab, pas de North.
- Gradient 40 K : chiffre du brief, dépend de la latitude polaire de référence.
- Choix de conception : rampe de 10 K, φ_H(Ω), f_min, D unique, noyau orographique précalculé. À marquer dans le code.

---

## Sources

- [V] climlab, Budyko Transport EBM : https://climlab.readthedocs.io/en/main/courseware/Budyko_Transport_EBM.html
- [V] climlab, modèles et équations : https://climlab.readthedocs.io/en/main/models.html
- [V] climlab, code EBM (A, B, D, a0, a2, T_f, pas de temps) :
  https://raw.githubusercontent.com/brian-rose/climlab/master/climlab/model/ebm.py
- [V] NASA NSSDC, fiches Terre / Vénus / Mars :
  https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html ;
  https://nssdc.gsfc.nasa.gov/planetary/factsheet/venusfact.html ;
  https://nssdc.gsfc.nasa.gov/planetary/factsheet/marsfact.html
- [S] Berger 1978, *J. Atmos. Sci.* 35(12) : 2362–2367 ; code :
  https://paleo.ltrr.arizona.edu/climate_forcing/orbital_variations/huybers2006insolation/daily_insolation.m
- [S] UW, handout EBM (A = 203,3, B = 2,09) : https://earthweb.ess.washington.edu/roe/Web_page_314/ESS314/Week_5_files/LabEBM_13.pdf
- [S] UW Bitz, modèle saisonnier (North & Coakley 1979) : https://atmos.uw.edu/~bitz/model/model_instructions/node1.html
- [S] Cornell, devoir modèle (Q, A, B, C) : https://e.math.cornell.edu/people/belk/modeling/Homework8.htm
- [S] McGehee & Widiasih (2012), albédos et T_f :
  https://www-users.cse.umn.edu/~mcgehee/publications/McGeheeWidiasih2012/index.html
- [S] IPCC TAR WG1 ch. 6, forçage CO2 (Myhre 1998) : https://archive.ipcc.ch/ipccreports/tar/wg1/222.htm
- [S] Science of Doom, doublement CO2 : https://scienceofdoom.com/2010/02/19/co2-an-insignificant-trace-gas-part-seven-the-boring-numbers/
- [S] Vömel, formule de Bolton : https://cires1.colorado.edu/~voemel/vp.html
- [S] Pyrcel, e_s (0,1 %) : https://pyrcel.readthedocs.io/en/latest/generated/pyrcel.thermo.es.html
- [S] Held & Hou 1980, notes NOAA : https://www.aoml.noaa.gov/ftp/hrd/annane/prelim_notes/2010_1_Hadley.pdf
- [S] Held & Hou 1980, notes Harvard EPS231 :
  https://courses.seas.harvard.edu/climate/eli/Courses/EPS231/Sources/08-Equable_Climate/1-equator-to-pole-Hadley/notes-Hadley-Held-Hou-1980-from-Vallis-AOFD-book.pdf
- [S] UCI, circulation générale (largeur de cellule, capacité du mélangé) :
  https://www.ess.uci.edu/~yu/class/ess228/lecture.11.general-circulation.2017.all.pdf
- [S] Zonal-mean dynamics, planètes en rotation lente (arXiv 1806.10494) : https://arxiv.org/pdf/1806.10494
- [S] Kottek et al. 2006 : https://www.doi.org/10.1127/0941-2948/2006/0130
- [S] Peel, Finlayson & McMahon 2007, *HESS* 11 : https://hess.copernicus.org/articles/11/1633/2007/
- [S] Smith & Barstad 2004, *J. Atmos. Sci.* 61 (non relu) ; code :
  https://github.com/fastscape-lem/orographic-precipitation
- [S] ExoPlaSim (2021) : https://ar5iv.labs.arxiv.org/html/2107.07685 ; docs : https://exoplasim.readthedocs.io
- [S] Laskar et al. 2004, *A&A* 428 : https://www.aanda.org/articles/aa/full/2004/46/aa1335/aa1335.fig.html
- [S] Universe Sandbox (AGU 2014), EBM 1D en temps réel dans un jeu :
  https://agu.confex.com/agu/fm14/webprogram/Paper19191.html
- [S] SERC, « Energy Balance Climate Model » : https://serc.carleton.edu/introgeo/mathstatmodels/examples/GEBM1.html
