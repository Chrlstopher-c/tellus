# Nutriments et cycles biogéochimiques pour Tellus

Recherche documentaire, 2026-11-01. Non validée par simulation.
Légende : **[V]** = titre et URL retrouvés pendant cette recherche ; **[M]** = tiré de la mémoire, à vérifier avant
codage ; **[C]** = calculé ici à partir d'autres valeurs.

---

## Recommandation

1. **Boîtes couplées à flux conservés** : chaque élément (C, N, P, O, H2O) a des stocks par cellule et des réservoirs
   globaux. Un processus ne fait que transférer une quantité d'un stock vers un autre.
2. **Deux échelles de tick** : biologie à **1 an** (NPP, respiration, décomposition, fixation N) ; géologie en
   **super-tick de 100 à 1 000 ans** (altération, volcanisme, enfouissement) avec flux moyennés.
3. **Intégration exacte des réservoirs linéaires** et **flux limités au stock disponible** : positivité et stabilité
   sans contrainte forte sur Δt.
4. **Liebig** (minimum d'eau, lumière, N, P) ; demande N et P par biome. Redfield 106:16:1 seulement pour le plancton.
5. **Thermostat long terme** : altération des silicates en fonction de pCO2 et de T (Walker-Hays-Kasting, GEOCARB).
   C'est la seule rétroaction qui ramène le CO2 sur 10^5 ans.
6. **Oxygène** : source = enfouissement net de carbone organique et de pyrite ; puits = gaz volcaniques réducteurs et
   altération oxydante. Départ à O2 = 0.
7. **Précision** : totaux en f64 sur CPU ; sur GPU (f32), sommes à ordre fixe ou compensées (Kahan), contrôle CPU.
8. **Pas d'écologie individu-centrée** dans le noyau chimique (Madingley trop coûteux par cellule). Modèle agrégé
   par cellule et par biome, inspiré de LPJ et CASA.
9. **Stocks visés après stabilisation** : atmosphère 600 à 900 GtC, sols 1 500 à 2 400 GtC, océan ≈ 38 000 GtC,
   végétation 450 à 650 GtC.
10. **Validation** : conservation à 1e-6 relatif par tick et par élément ; positivité ; pas de dérive d'un ordre de
    grandeur sur 10 000 ans sans perturbation.

---

## 0. Conversions utiles [C]

| Grandeur | Équivalence |
|---|---|
| 1 ppm de CO2 atmosphérique | ≈ 2,124 GtC |
| 1 GtC | ≈ 83,3 Tmol C ; 1 TgN ≈ 71,4 Gmol N ; 1 GtP ≈ 32,3 Tmol P |
| 1 km³ d'eau | ≈ 1 Gt |
| O2 atmosphérique | ≈ 3,7 × 10^7 Tmol (≈ 21 % en mol) |

---

## 1. Cycle de l'eau

### 1.1 Réservoirs

| Réservoir | Volume (km³) | Part |
|---|---|---|
| Océans | 1,338 × 10^9 | ≈ 96,5 % |
| Glaces | 24 × 10^6 | ≈ 1,7 % |
| Eaux souterraines | ≈ 23 × 10^6 | ≈ 1,7 % |
| Lacs, humidité du sol, atmosphère, rivières | 0,09 × 10^6 ; 0,017 × 10^6 ; 13 × 10^3 ; 2 × 10^3 | < 0,01 % chacun |

Valeurs Shiklomanov (USGS) [M] ; la page USGS testée a répondu 404, URL à retrouver. Ratio océan ≈ 97 % confirmé par
Trenberth et al. 2007, *Journal of Hydrometeorology* 8, 758-769, https://doi.org/10.1175/JHM600.1 [V] (texte non lu).

### 1.2 Flux annuels [M]

- Évaporation ≈ 505 000 km³/an (océan ≈ 434 000, continent ≈ 71 000). Précipitations de même ordre.
- Ruissellement continental ≈ 40 000 km³/an. Séjour atmosphérique ≈ 9 à 10 jours.
- Clausius-Clapeyron : ≈ +7 % de vapeur saturante par kelvin ; c'est le retour principal du cycle sur le climat.

### 1.3 Règles Tellus (eau)

- Par cellule : `P = pluie + neige` ; fonte par degré-jour ; `ET = min(PET, eau disponible)` avec PET de Hamon ou
  Priestley-Taylor [M].
- Sol saturé → ruissellement ; reste → drainage vers la nappe ; nappe → rivière ou océan.
- Glaces : accumulation − ablation ; vêlage vers l'océan au-delà d'un seuil d'altitude.
- Unités : km³ (1 km³ = 1 Gt). Toute eau sort d'un stock pour entrer dans un autre.

---

## 2. Cycle du carbone

### 2.1 Stocks et flux de référence

| Réservoir | Stock | Source |
|---|---|---|
| Atmosphère | ≈ 890 GtC (2023, 419 ppm) ; ≈ 590 GtC à 280 ppm | [V] GCB 2024 ; [C] |
| Végétation | 450 à 650 GtC | [M] Ciais et al. 2013 (IPCC AR5 WG1 ch. 6) |
| Sols | 1 500 à 2 400 GtC | [M] Jobbágy & Jackson 2000 |
| Océan (DIC) | ≈ 38 000 GtC | [M] Ciais et al. 2013 |
| Carbone fossile | ≈ 4 000 GtC | [M] |
| Carbonates sédimentaires | ≈ 6 × 10^7 GtC | [M] Falkowski et al. 2000, *Science* |

**Budget 2023** [V] Friedlingstein et al. 2025, *ESSD* 17, 965, https://essd.copernicus.org/articles/17/965/2025/ :
émissions fossiles ≈ 10,1 GtC + usage des sols ≈ 1,0 ; croissance atmosphérique ≈ 5,9 ; puits océan ≈ 2,9 ; puits
continent ≈ 2,3 ; résidu ≈ 0.

**Flux naturels** [M] : GPP continentale ≈ 120 GtC/an (Beer et al. 2010, *Science*, DOI 10.1126/science.1184984) ;
NPP continentale ≈ 55 à 60 ; respiration hétérotrophe ≈ 50 à 55 ; NPP océanique ≈ 50 ; échange brut océan-atmosphère
≈ 90 dans chaque sens ; export vers la profondeur ≈ 5 à 10 ; volcanisme ≈ 0,1 à 0,3 ; enfouissement organique
≈ 0,1 à 0,2 ; carbonates ≈ 0,2 GtC/an.

### 2.2 Photosynthèse (NPP)

**Miami** (Lieth 1975, *Primary Productivity of the Biosphere*, Springer) [M] :
- `NPP_T = 3000 / (1 + exp(1,315 − 0,119 T))`, T en °C ; `NPP_P = 3000 × (1 − exp(−0,000664 P))`, P en mm/an.
- `NPP = min(NPP_T, NPP_P)` en g de matière sèche m⁻² an⁻¹ ; × 0,45 à 0,5 pour le carbone.
- Contrôle : T = 20 °C, P = 1 000 mm → ≈ 1 450 g m⁻² an⁻¹, ordre forêt tropicale.
- Pas de réponse au CO2 : ajouter `f_CO2 = 1 + β_CO2 ln(C/C0)`, β_CO2 ≈ 0,3 à 0,5 [M].

**CASA** (Potter et al. 1993, *Global Biogeochemical Cycles*, DOI 10.1029/93GB02725) [M] :
`NPP = ε × FPAR × PAR × T_ε1 × T_ε2 × W_ε`, ε ≈ 0,4 à 0,6 gC MJ⁻¹. Version mensuelle si la saisonnalité compte.

**Autotrophe** : `R_a ≈ 0,4 à 0,6 × GPP` [M] ; `GPP = NPP + R_a`.

**LPJ** (Sitch et al. 2003, *Global Change Biology* 9, 161-185, DOI 10.1046/j.1365-2486.2003.00569.x) [V] : référence
pour la structure des flux végétation-sol-eau (pas journalier, structure annuelle).

### 2.3 Respiration et Q10

- `R_h = R_ref × Q10^((T − T_ref)/10)`, Q10 ≈ 2 [M]. À basse température : Lloyd & Taylor 1994, *Functional Ecology*
  8, 315-323, DOI 10.2307/2389824 [M].
- Facteur d'humidité entre capacité au champ et flétrissement [M].
- Appliquer Q10 sur la température **pondérée par la courbe intra-annuelle**, pas sur la moyenne (inégalité de Jensen,
  écart ≈ 10 % sur R_h en climat contrasté).

### 2.4 Sols

RothC (Coleman & Jenkinson 1996) [M] : pools DPM, RPM, BIO, HUM ; constantes annuelles k ≈ 10 ; 0,3 ; 0,66 ; 0,02 an⁻¹ ;
partition CO2 / pools ≈ 46 % vers BIO et HUM. Alternative plus riche en N et P : CENTURY (Parton et al. 1987, *SSSAJ*) [M].

### 2.5 Océan-atmosphère et pompe

- `F = k_w × K0 × (pCO2_atm − pCO2_surf)`, `k_w ≈ 0,31 u10²` (cm/h, Wanninkhof 1992) [M].
- Il faut DIC **et** alcalinité pour un tampon de Revelle correct [M].
- Couche de mélange : équilibre en ≈ 1 an [M] → un tick annuel suffit.
- Export `= e_ratio × NPP_océan`, e_ratio ≈ 0,1 à 0,2 [M] ; remise en solution dans la colonne, fraction qui atteint le fond selon la profondeur.

### 2.6 Volcanisme et enfouissement

Dégazage ≈ 0,1 à 0,3 GtC/an [M], paramètre tectonique (pas de rétroaction directe). L'enfouissement organique et
carbonaté sort le carbone du cycle court terme.

### 2.7 Thermostat de Walker-Hays-Kasting

Walker, Hays & Kasting (1981, *Journal of Geophysical Research*) [V] : plus de CO2 → plus chaud → altération plus rapide
→ CO2 précipité en carbonates → retour à l'équilibre. Forme retenue [M] :

```
F_sil = F_sil0 × (pCO2/pCO2_0)^n × exp(β (T̄ − T0)) × g(Q̄) × A_exposée
```

- n ≈ 0,3 à 0,5 (GEOCARB : Berner, Lasaga & Garrels 1983, *Am. J. Sci.* 283 ; Berner 2004, OUP) [M].
- β ≈ 0,1 K⁻¹ (Arrhenius, Ea ≈ 60 à 90 kJ/mol) [C] ; une revue [V] donne ≈ 15 %/°C, compatible.
- g(Q) ≈ linéaire en ruissellement ; A_exposée lié au relief de la cellule [M].
- Limite : Brantley et al. 2023, *Science* [V], estiment que l'érosion et le carbone organique ont modifié les réglages
  du thermostat au fil de la dérive des continents. C'est une approximation pour une planète donnée.

**Effet de serre** : `F = 5,35 ln(C/C0)` W m⁻² (Myhre et al. 1998, GRL, DOI 10.1029/98GL01908) [M] ; sensibilité ≈ 3 K par
doublement (IPCC AR6) [M].

---

## 3. Cycle de l'azote

| Terme | Ordre de grandeur | Source |
|---|---|---|
| N2 atmosphérique | ≈ 3,9 × 10^6 GtN (quasi constant) | [C] |
| Fixation terrestre naturelle, préindustrielle | 100 à 290 TgN/an | [V] Galloway et al. 2004, *Biogeochemistry* 70, 153-226 (PDF USC, lu partiellement) |
| Haber-Bosch | ≈ 100 à 120 TgN/an | [M] Galloway et al. 2008 |
| Fixation marine | ≈ 100 à 200 TgN/an | [M] Gruber & Galloway 2008, *Nature*, DOI 10.1038/nature06592 |
| Foudre | ≈ 2 à 8 TgN/an | [M] Schumann & Huntrieser 2007, *ACP*, DOI 10.5194/acp-7-3823-2007 |
| Dénitrification | ≈ 100 à 300 TgN/an | [M] Gruber & Galloway 2008 |
| Nitrate océanique | ≈ 570 GtN | [C] : 30 µmol/kg × 1,35 × 10^21 kg |

**Modèle** :
- Demande : `N_demande = NPP / R_CN`, R_CN (masse) ≈ 1/30 à 1/100 selon le tissu [M]. Redfield plancton : N:C ≈ 0,151 [C].
- Fixation biologique facultative : déclenchée par le déficit en N, coût énergétique réduisant la NPP [M].
- Foudre : fonction de la convection, vers N_min [M].
- Nitrification (N_org → NH4+ → NO3−) et lessivage de NO3− avec le drainage [M].
- Dénitrification : croît avec l'humidité et la faible teneur en O2 ; retour vers N2 atmosphérique.
- Règle : le N2 atmosphérique est un réservoir négligeable à l'échelle de Tellus. Sans rétroaction de fixation sur le déficit,
  les écosystèmes ont un azote infini (artefact).

---

## 4. Cycle du phosphore

Pas de phase gazeuse. Cycle lent, issu de l'altération de l'apatite.

- Flux fluvial : 0,26 à 0,92 Tmol P/an (≈ 8 à 28 TgP/an), issu de Ruttenberg (*Treatise on Geochemistry*, ch. 8.13) et
  Slomp (2011), cité dans une prépublication ESD 2016 [V]. PDF : https://faculty.uml.edu/David_Ryan/84.653/documents/RuttenbergGlobalPCycle.pdf
- Stock océanique de phosphate : ≈ 2 µmol/kg × 1,35 × 10^21 kg ≈ 90 GtP [C].
- Temps de séjour océanique ≈ 10^4 à 10^5 ans [M].

**Modèle** :
- `F_P,alt = k_alt × P_apatite × f(T, Q, relief)` ; stock apatite fini, déplété sur 10^6 ans.
- Sols : P labile ↔ P occlus, constante qui augmente avec l'âge du sol [M].
- Lessivage : P dissous avec le drainage ; une partie retenue par Fe et Al oxydés [M].
- Long terme : le P limite la productivité sur 10^6 ans car l'apport d'apatite ne suit pas la fixation N (Tyrrell 1999,
  *Nature*, DOI 10.1038/21358) [M].

---

## 5. Oxygène atmosphérique

- Stock ≈ 3,7 × 10^7 Tmol O2 ≈ 21 % en mol [C].
- Grand Événement d'Oxydation vers 2,4 Ga [M] : Lyons, Reinhard & Planavsky 2014, *Nature* 506, 307-315,
  https://doi.org/10.1038/nature13068 [V] (date non confirmée par le texte lu).
- Oxygénique antérieur : Planavsky et al. 2014, *Nature Geoscience*, DOI 10.1038/ngeo2122 [V].

**Bilan** (photosynthèse et respiration s'annulent à l'annuel) :

```
dO2/dt = F_enfouissement_org(C) + F_enfouissement_pyrite(S)
       − F_réducteurs_volcaniques(H2, S) − F_altération_oxydante(Fe2+, S2−) − F_photochimique
```

- Enfouissement organique 0,16 GtC/an ≈ 13 Tmol O2/an [C] : ordre de la source nette actuelle.
- O2 part de 0 dans Tellus. La transition dépend des réducteurs volcaniques et de roche plus que de la photosynthèse.
- Seuil d'ozone ≈ 10^-5 PAL [M] (Kasting 1993, *Science*, DOI 10.1126/science.11536547). Règle : colonisation terrestre
  bloquée sous ce seuil.

---

## 6. Liebig et Redfield

- `NPP = NPP_potentiel × min(f_N, f_P, f_eau, f_T, f_lum)`, chaque f ∈ [0, 1].
- Plus doux et plus réaliste : limitation Michaelis-Menten par nutriment, puis minimum ou moyenne géométrique [M].
- Redfield : C:N:P ≈ 106:16:1 molaire pour le plancton [V] (https://en.wikipedia.org/wiki/Redfield_ratio), soit
  ≈ 41:7,2:1 en masse [C]. Attribution contestée (1934 ou 1958) [V].
- Origine du rapport toujours débattue : Arrigo 2005, *Nature* [V]. N:P = 16 est une moyenne, pas un optimum.
- Écarts supérieurs à Redfield dans les gyres pauvres en nutriments (Jones et al. 2024, prépublication) [V].
- **Règle Tellus** : Redfield est un défaut océanique. Les biomes terrestres ont leur propre C:N et C:P
  (bois ≈ 100 à 500 en C:N, feuilles ≈ 20 à 40) [M].

---

## 7. Modèles de référence

| Modèle | Usage pour Tellus |
|---|---|
| BLAG / GEOCARB (Berner et al. 1983 ; Berner 2004) [M] | Lois pCO2 et T de l'altération, forme des rétroactions |
| COPSE (Bergman et al. 2004, *Am. J. Sci.*) [M] | Boîtes couplées C, O2, S, P, N : structure à reprendre |
| Walker-Hays-Kasting (1981) [V] | Thermostat silicates |
| Miami (Lieth 1975) [M] | NPP à l'équilibre par cellule : première version |
| CASA (Potter et al. 1993) [M] | NPP mensuelle si saisonnalité explicite |
| LPJ (Sitch et al. 2003) [V] | Structure des flux végétation-sol-eau |
| Madingley (Harfoot et al. 2014, *PLoS Biology*, DOI 10.1371/journal.pbio.1001841) [V] | Référence pour les espèces (phase 4). Hors noyau chimique. |
| RothC / CENTURY [M] | Carbone et azote du sol |

Choix : boîtes à la BLAG/COPSE pour la géochimie, Miami puis CASA simplifié pour la NPP, RothC pour les sols, LPJ comme
repère de structure, Madingley comme référence pour le vivant.

---

## 8. Pas de temps et agrégation

- **Réservoir linéaire** `dS/dt = I − kS` : intégration exacte `S(t+Δt) = S·e^(−kΔt) + (I/k)(1 − e^(−kΔt))`. Stable quel
  que soit Δt. En explicite, `kΔt < 1` est requis [M].
- **Flux limité** : `F = min(F_demandé, S_source)` avant transfert. Positivité garantie.
- **Séparation des échelles** : biologie (1 an) et géologie (100 à 1 000 ans) en splitting de Strang ; couplage par flux moyennés.
- **Agrégation** : Q10, Miami et Michaelis-Menten évalués sur des températures et précipitations **intra-annuelles**, puis
  moyennés. Solution : 12 sous-pas mensuels avec climatologie précalculée.
- **Géologie** : erreur acceptable tant que pCO2 varie de moins de 5 % sur le super-tick [M].
- **Déterminisme** : graine fixe et ordre de sommation fixe. Sur GPU, réductions non déterministes → contrôle CPU f64.

---

## Règles pour Tellus

### R.1 Variables d'état par cellule (40 000 à 160 000 cellules)

Cellules terrestres :

| Nom | Unité | Type | Rôle |
|---|---|---|---|
| `T_moy`, `P_an` | °C, mm/an | lecture | Climat (module climat) |
| `w_sol`, `neige`, `nappe` | mm | f64 | Eau du sol, neige, eau souterraine |
| `glace` | km³ | f64 | Glace continentale |
| `C_veg`, `C_litiere` | gC m⁻² | f64 | Carbone végétal, litière |
| `C_DPM`, `C_RPM`, `C_BIO`, `C_HUM` | gC m⁻² | f64 | Pools du sol (RothC) |
| `N_veg`, `N_org`, `N_min` | gN m⁻² | f64 | Azote végétal, organique, minéral |
| `P_veg`, `P_org`, `P_lab`, `P_occ`, `P_apatite` | gP m⁻² | f64 | Phosphore et stock altérable |
| `biome` | énum | u8 | Stœchiométrie, paramètres NPP |
| `LAI`, `f_couvert` | — | f32 | Couvert (rendu, PAR) |

Cellules océaniques :

| Nom | Unité | Type | Rôle |
|---|---|---|---|
| `h_mix` | m | f64 | Couche de mélange |
| `DIC`, `ALK` | mol m⁻³ | f64 | Carbone inorganique, alcalinité |
| `DIN`, `DIP`, `O2_dis` | mol m⁻³ | f64 | Azote, phosphore, oxygène dissous |

### R.2 Variables globales

| Nom | Unité | Rôle |
|---|---|---|
| `C_atm` | GtC | Carbone atmosphérique ; pCO2 (ppm) = C_atm / 2,124 |
| `O2_atm` | Tmol | Oxygène atmosphérique |
| `N2_atm` | GtN | Constant, pour la comptabilité |
| `H2O_océan`, `H2O_glace`, `H2O_nappe`, `H2O_atm` | km³ | Stocks d'eau |
| `C_prof`, `DIN_prof`, `DIP_prof`, `O2_prof` | GtC, GtN, GtP, Tmol | Océan profond |
| `C_carb`, `C_org_sed` | GtC | Carbonates, carbone organique enfoui |
| `P_roche`, `N_roche` | GtP, GtN | Réserves de roche |
| `F_volc_C`, `F_volc_red` | GtC/an, Tmol e⁻/an | Volcanisme |
| `T_glob` | °C | Température moyenne pondérée par la surface |
| `t_annee` | an | Horloge |

### R.3 Pseudo-code

```
fonction tick_annuel(t):
    pour chaque cellule terrestre i, pour chaque mois m (12 sous-pas) :
        npp_pot ← miami_ou_casa(T[i,m], P[i], lumière[i,m], C_atm)
        npp     ← min(npp_pot, N_min[i]·R_CN(biome), P_lab[i]·R_CP(biome))      # Liebig
        gpp     ← npp + R_a
        transfert(C_atm → C_veg[i], gpp·dt/12)                    # limité à C_atm
        transfert(C_veg[i] → C_atm, R_a·dt/12)                    # respiration autotrophe
        transfert(C_veg[i] → C_litiere[i], mort·C_veg[i]·dt/12)
        r_h ← R_ref · Q10^((T[i,m]−10)/10) · f_eau(w_sol[i]) · C_litiere[i]
        transfert(C_litiere[i] → C_atm, r_h·fraction_CO2)          # reste → C_DPM / C_RPM
    pour chaque cellule océanique j :
        transfert(C_atm → DIC[j], k_w·K0·(pCO2_atm − pCO2_surf(DIC[j], ALK[j], T, S)))
        npp_oc ← min(Φ_oc·lumière, DIN[j]·R_NC, DIP[j]·R_PC)
        transfert(DIC[j] → C_prof, e_ratio·npp_oc·f_fond(h_mix[j]))
    verifier_conservation(t)

fonction tick_geologique(t, Δt_géo):                              # tous les 100 à 1 000 ans
    F_sil ← F_sil0 · (pCO2/pCO2_0)^n · exp(β(T_glob − T0)) · g(Q̄)
    transfert(C_atm → C_carb, F_sil·Δt_géo)
    transfert(F_volc_C·Δt_géo → C_atm)
    transfert(C_atm/ocean → C_org_sed, enfouissement_org·Δt_géo)
    O2_atm += (enfouissement_org + pyrite − F_volc_red − F_oxydant)·Δt_géo
    P_roche, P_apatite ← altération avec exp(−k·Δt_géo)
    verifier_conservation(t)
```

### R.4 Règles de conservation

- Un transfert est la seule primitive qui modifie un stock. Écriture directe sur un stock = interdit.
- Tout transfert est limité : `F ← min(F, S_source)`. Un clamp silencieux est compté et signalé.
- Par élément : `Σ(t+1) − Σ(t) = entrées − sorties enregistrées aux frontières`.
- Totaux en f64 ; contrôle GPU recalculé en CPU à chaque tick de contrôle.

---

## Tests de validation

### T.1 Conservation de masse

- Par tick et par élément (C, N, P, O, H2O) : `|Σ(t+1) − Σ(t) − bilan_frontières| / Σ(t) < 1e-6`.
- Sur 10 000 ans sans perturbation : dérive cumulée < 1e-6 × Σ(0).
- Test unitaire : une cellule isolée, flux aléatoires à graine fixe, 10 000 ticks, bilan exact.
- Positivité : tout stock ≥ 0 à chaque pas ; clamps comptés, alerte au-delà de 1e-9 relatif.

### T.2 Ordres de grandeur après stabilisation (planète tempérée de référence, 10 000 ans, sans forçage brutal)

| Grandeur | Cible | Tolérance | Source |
|---|---|---|---|
| CO2 atmosphérique | 280 à 420 ppm (590 à 890 GtC) | ± 30 % | [V] GCB 2024 ; [C] |
| Végétation | 450 à 650 GtC | ± 30 % | [M] Ciais 2013 |
| Sols | 1 500 à 2 400 GtC | ± 30 % | [M] Jobbágy & Jackson 2000 |
| Océan (DIC) | ≈ 38 000 GtC | ± 5 % | [M] Ciais 2013 |
| NPP continentale | 50 à 70 GtC/an | ± 30 % | [M] |
| NPP océanique | 45 à 60 GtC/an | ± 30 % | [M] |
| Enfouissement organique | 0,1 à 0,2 GtC/an | ± 50 % | [M] |
| pO2 après GOE | 0,1 à 0,3 bar | ordre de grandeur | [M] |
| Nitrate océanique | ≈ 570 GtN | ± 20 % | [C] |
| Phosphate océanique | ≈ 90 GtP | ± 20 % | [C] |

### T.3 Tests de comportement

- **WHAK** : doubler le volcanisme ⇒ hausse de pCO2, puis retour partiel. Sur 10 000 ans, seul le sens est testable.
- **Effet de serre** : +1 K ⇒ NPP et R_h changent dans le bon sens, d'ordre quelques %.
- **Liebig** : retirer N ⇒ NPP baisse, la fixation compense avec un délai.
- **Redfield** : export C:N:P à ±20 % de 106:16:1 pour un plancton stable.
- **Oxygène** : O2 = 0 au départ ; aucune hausse sans enfouissement organique ou pyrite non nul.
- **Stabilité numérique** : doubler le pas géologique (100 → 200 ans) ne modifie pas les stocks de plus de 5 %.

### T.4 Avant codage

- Retrouver dans la source toutes les valeurs **[M]** (Miami, GEOCARB, constantes RothC, flux N et P).
- Lire le texte intégral de Galloway 2004, Ruttenberg 2004, Trenberth 2007.
- Ajouter la luminosité stellaire croissante : elle fixe le pCO2 d'équilibre.
