# Vivant : écologie et évolution émergentes pour Tellus
Date : 2026-11-01. Lu : `PROJET.md`, `ARCHITECTURE.md`, `STATE.md`, `TODO.md`. Périmètre : `simulation/src/life/`, branchements avec `climate/` et `nutrients/`.

**Légende.** [S] = lu ou trouvé par recherche, URL en fin de document. [M] = connaissance de référence, non revérifiée ici : à confirmer avant implémentation. [D] = décision de conception proposée pour Tellus.

**Limites de cette recherche.** Le papier Brown 2004 n'a pas pu être ouvert (erreur de certificat). Madingley a été lu sur les 100 000 premiers caractères d'un texte de 167 512, sans les tables de paramètres. Les équations de Miami viennent d'un PDF agronomique, non du papier de Lieth.

## Recommandation
1. **Base : l'architecture de Madingley** (Harfoot et al. 2014) : cellules, cohortes d'hétérotrophes, métabolisme en loi de puissance, prédation, mortalité en trois sources, dispersion [S]. Sans ses groupes fonctionnels figés : Madingley n'évolue pas, ses traits sont fixés à l'initialisation.
2. **Génome = distribution de traits par lignée** (moyenne et variance, modèle des moments), pas une séquence. Mutation, sélection, dérive et spéciation en découlent [D].
3. **Espèce = lignée** : un identifiant attribué à chaque scission, jamais codé à la main. Une lignée est vivante si elle a au moins une cohorte [D].
4. **Tick écologique = 1 an**, mise à jour échelonnée (1/10 des cellules par tick), sous-pas si `r·Δt ≥ 1` [D]. Madingley tourne au mois : à Tellus, c'est environ 15 000 fois plus de travail par année simulée (section 4.4).
5. **Métabolisme ancré sur la théorie métabolique (MTE)** : taux ∝ M^(3/4)·exp(−E/kT), E = 0.65 eV, k = 8.617×10⁻⁵ eV/K [S, M]. Une seule source de température pour tout le vivant.
6. **Prédation** : Holling III pour les prédateurs (stabilisant, comme chez Madingley [S]), Holling II pour les herbivores [D]. Transfert d'énergie de 10 % par niveau [M].
7. **Végétation** : NPP de Miami comme plafond climatique [S], 5 types fonctionnels de plantes en compétition de type Tilman [M], feu stochastique simple [D].
8. **Dispersion** : diffusion discrète conservative sur le graphe des cellules, avec noyau à queue épaisse (Kot et al. 1996 [S]), pas un noyau gaussien pur.
9. **Grandes transitions** (abiogenèse, photosynthèse, eucaryotes, multicellularité, sortie des eaux) : probabilité par cellule et par an, conditionnée à l'état du monde, jamais à une date [D].
10. **Ordre de développement** : une cellule, puis 1 000 cellules, puis la planète. Chaque règle est testée seule (Lotka-Volterra, Kleiber, Fisher) avant couplage.
11. **Ne rien copier** d'Avida, Tierra, Polyworld, Thrive, Equilinox ou Species : on retient des mécanismes (section 9).
12. **Point à trancher avec Chris** : 10 000 ans suffisent-ils pour faire émerger la vie, ou seulement pour la tenir bornée une fois là ? Voir section 7.0.

## 1. Biomes et végétation
### 1.1 Diagramme de Whittaker
Biome = f(température annuelle moyenne T, précipitations annuelles P) [M, Whittaker 1975]. Seuils approximatifs, à utiliser en validation et non en table : désert P ≲ 250 mm/an ; toundra T ≲ 0 °C ; forêt tropicale humide T ≳ 24 °C et P ≳ 2 000 mm/an [M].

Règle Tellus [D] : le biome n'est jamais stocké. C'est la classe de Whittaker calculée à partir de l'état (T, P, NPP, type dominant). Il sert de test (section Tests), pas d'entrée.

### 1.2 Production primaire nette : modèle de Miami (Lieth 1975)
Équations [S, PDF Embrapa](https://www.alice.cnptia.embrapa.br/alice/bitstream/doc/852804/1/ID410252009CBAcd217326.pdf) :
- NPP_T(T) = 3000 / (1 + exp(1,315 − 0,119·T)), T en °C, NPP en g de matière sèche m⁻² an⁻¹ ;
- NPP_P(P) = 3000 · (1 − exp(−0,000664·P)), P en mm/an ;
- NPP = min(NPP_T, NPP_P).

Contrôle de cohérence (calcul) : T = 20 °C et P = 1 000 mm donnent NPP_T ≈ 2 231, NPP_P ≈ 1 455, donc NPP = 1 455 g m⁻² an⁻¹. T = 0 °C donne NPP_T ≈ 635.

Limites [M] : moyennes annuelles, pas de CO₂, pas de sol, pas d'azote ni de phosphore. Usage Tellus [D] : plafond climatique de la NPP et valeur d'initialisation. La NPP réelle prend le minimum avec l'azote et le phosphore (loi du minimum de Liebig [M]).

Ordre de grandeur global [M, Field et al. 1998] : production primaire totale d'environ 100 PgC/an, dont 56 en continental et 48 en océanique.

### 1.3 Types fonctionnels de plantes (PFT)
LPJ utilise 10 PFT définis par des traits physiologiques, morphologiques, phénologiques, bioclimatiques et de réponse au feu. Leurs couvertures évoluent chaque année par compétition et feu. Photosynthèse et eau sont calculées au pas journalier, la structure au pas annuel [S, Sitch et al. 2003](https://impacts.ucar.edu/en/publications/evaluation-of-ecosystem-dynamics-plant-geography-and-terrestrial-/).

Proposition Tellus, 5 PFT et plancton [D] :

| PFT | Traits clés | Rôle |
|---|---|---|
| Herbe | croissance rapide, forte dispersion, bonne tolérance au feu | pionnier |
| Arbuste | croissance moyenne, tolérance à la sécheresse | succession |
| Arbre tempéré feuillu | ombrage moyen, tolérance au froid modérée | climax tempéré |
| Arbre tropical | T et P élevées, ombrage fort | climax tropical |
| Conifère | froid, sol pauvre, résistance au feu variable | boréal |
| Plancton (océan) | nutriments, lumière, dispersion par courants | base marine |

Chaque PFT porte : T_opt, largeur thermique, P_min, tolérance au feu, tolérance à l'ombre, vitesse de colonisation, taux de croissance, coût d'entretien. Ces valeurs sont des traits de départ, avec dérive possible si on veut qu'elles évoluent (section 5).

### 1.4 Succession
Modèle compétition-colonisation de Tilman (1994) [M] pour la fraction de couverture C_i de chaque PFT i, ordonnés par compétitivité :

dC_i/dt = c_i · C_i · (1 − Σ_{j≤i} C_j) − m_i · C_i

- c_i = colonisation (élevée pour les pionniers, faible pour les climax) ;
- m_i = mortalité (perturbation, feu, âge).

Les climax dominent quand les pionniers disparaissent. Le feu ou une perturbation remet le compteur à zéro. Aucune cascade n'est codée.

Variante si une structure forestière plus fine est nécessaire : modèle de trouées (gap model, JABOWA, Botkin et al. 1972 [M]).

### 1.5 Feu
SPITFIRE [S, Thonicke et al. 2010, doi:10.5194/bg-7-1991-2010] : feu = ignition (foudre et humaine) × propagation (Rothermel) × humidité du combustible, couplé à LPJ. Une recherche de 2025 signale qu'une mauvaise implémentation de Rothermel introduit de gros biais de vitesse de propagation [S].

Version simplifiée Tellus [D] :

P_feu(i, an) = 1 − exp(−λ_ign · F_i · (1 − H_i) · Δt)

- F_i = charge de combustible (kg m⁻²), saturante : F = F_max · (1 − exp(−k · litière)) ;
- H_i = humidité du combustible ∈ [0, 1], dérivée du rapport P/ETP ;
- λ_ign : à calibrer sur les cibles de surface brûlée annuelle : savane 10–50 % [M], forêt boréale ≈ 1 % [M], forêt humide ≈ 0 [M] ;
- effet : perte de biomasse aérienne (herbe presque totale, arbres une fraction [M]), libération de nutriments, remise à zéro de la succession.

## 2. Métabolisme et allométrie
### 2.1 Loi de Kleiber
B = B₀ · M^(3/4) [M, Kleiber 1932]. L'exposant 3/4 est contesté (surface contre réseaux de distribution). Une revue de 2004 dans Functional Ecology relance le débat sans trancher [S, via recherche].

### 2.2 Théorie métabolique de l'écologie (MTE)
B = B₀ · M^β · exp(−E/kT) [S, Brown et al. 2004, Ecology 85:1771 ; Gillooly et al. 2001, Science 293:2248, [PubMed](https://pubmed.ncbi.nlm.nih.gov/11567137/)].

- β = 3/4 [S].
- E ≈ 0.65 eV, moyenne des processus respiratoires ; plage prédite 0.6–0.7 eV. Pour certains groupes, une valeur empirique plus basse (≈ 0.5 eV, poissons) [S, réponse Gillooly 2005].
- k = 8.617×10⁻⁵ eV/K ; T en kelvins [M].

Conséquences calculées :
- Entre 288 K et 298 K, le facteur est exp(E/k · (1/288 − 1/298)) ≈ 2,4 : Q10 ≈ 2,4 autour de 288 K.
- Le taux double environ tous les 7,6 K autour de 288 K.

### 2.3 Prédictions populationnelles
- Taux de mortalité m ∝ M^(−1/4) · exp(−E/kT) [M].
- Temps de génération τ ∝ M^(1/4) · exp(E/kT) [M].
- Densité d'équilibre N ∝ M^(−3/4), hypothèse d'équivalence énergétique (Damuth 1981, pente ≈ −3/4 chez les mammifères) [M].
- Taux de croissance intrinsèque r = b − m, donc r ∝ M^(−1/4) : les petits croissent vite.

Implémentation [D] : chaque cohorte reçoit b et m dérivés de M et T. Les constantes (B₀, facteur de mortalité) sont calibrées pour que la densité suive M^(−3/4) (test 8).

### 2.4 Reproduction selon la masse
Compromis r/K [D] : nombre de descendants × masse par descendant ≈ f, fraction de l'énergie adulte. f est un trait génétique. Fécondité = f · B / m_naissance.

Madingley [S] : la reproduction intervient quand la réserve atteint un seuil de la masse adulte. Les semelpares consacrent une fraction fixe de leur masse adulte.

### 2.5 Température
- Terrestre : les ectothermes ne sont actifs qu'entre leurs limites thermiques (traits génétiques CT_min et CT_max) ; les endothermes sont actifs en permanence [S, Madingley].
- Océan : les ectothermes sont actifs en permanence [S, Madingley].
- Coût d'entretien = B(T). Les coûts de chasse et de fuite s'ajoutent (section 8).

## 3. Dynamique des populations
### 3.1 Logistique et discrétisation
dN/dt = r·N·(1 − N/K) [M].

Euler explicite N' = N + r·N·(1 − N/K) est stable pour 0 < r·Δt < 2. Au-delà : oscillations, puis chaos (carte logistique) [M].

Règle [D] : Δt = 1 an, et r ≤ 1 pour les espèces lentes. Si r·Δt ≥ 1, sous-pas : Δt_sub = Δt / ⌈r·Δt⌉.

### 3.2 Lotka-Volterra
dN/dt = a·N − b·N·P ; dP/dt = c·N·P − d·P [M].

En continu, les orbites sont fermées. Euler explicite les fait diverger en spirale. Quantité conservée : V = c·N − d·ln N + b·P − a·ln P [M]. Intégrateur minimum : RK2. La dérive de V est un test (test 6).

### 3.3 Réponse fonctionnelle de Holling
- Type I : f(N) = a·N.
- Type II : f(N) = a·N / (1 + a·h·N). Saturation par temps de manipulation h [M, Holling 1959].
- Type III : f(N) = a·N² / (1 + a·h·N²). Seuil sigmoïde, proies protégées à faible densité, effet stabilisant [M].

Madingley utilise Type III. Son temps de manipulation croît linéairement avec la masse de la proie pour les carnivores, et décroît en puissance de la masse du prédateur pour les herbivores [S].

### 3.4 Efficacité trophique
Lindeman (1942) [M] : environ 10 % de l'énergie passe d'un niveau au suivant, avec une plage de 5 à 20 %.

Tellus [D] : assimilation α (≈ 0,5–0,8 selon le régime [M]) × efficacité de production (≈ 0,1–0,2 [M]). Le prédateur gagne e · B_consommée, avec e = 0,1. Le reste part en respiration, c'est une perte, non un stock.

### 3.5 Réseau trophique
Madingley [S] : le choix des proies suit une gaussienne en log-masse, centrée sur une taille de proie optimale proportionnelle à la masse du prédateur. Traits : position de l'optimum et largeur, tous deux dans le génome.

Le réseau émerge des optimums. Aucune chaîne n'est codée. La pyramide émerge si e ≈ 0,1 et si les prédateurs sont en moyenne 10 à 100 fois plus gros que leurs proies [D].

### 3.6 Dispersion et diffusion
Diffusion discrète sur le graphe des cellules [D] :

flux_ij = D · (N_i − N_j) / ℓ²

où ℓ est la distance entre centres. Stabilité explicite : D·Δt/ℓ² ≤ 1/4 sur une maille carrée à 4 voisins [M]. Sur maillage irrégulier, flux conservatif et Δt borné par le plus petit ℓ.

Fisher-KPP : onde progressive de vitesse minimale c = 2√(rD) [M]. Kot, Lewis et van den Driessche (1996) montrent que les noyaux de dispersion réels sont à queue épaisse, et qu'un noyau gaussien sous-estime fortement la vitesse d'invasion [S].

Noyau mixte [D] : fraction 1 − q en diffusion locale, fraction q (≈ 10⁻³ par an) en sauts sur quelques cellules. Les effets Allee cassent la formule de vitesse simple [S, via recherche].

### 3.7 Stabilité
May (1972) [M] : un grand système aléatoire complexe devient instable au-delà d'un seuil de connectance × force d'interaction. Remèdes utilisés ici : Holling III (refuges), densité-dépendance, dispersion, plafonds.

Gardes [D] :
- mortalité ≤ 0,5 par tick ;
- abondance bornée à ≥ 0 ;
- masse conservée à chaque tick : le clamp est comptabilisé comme une perte, jamais effacé.

## 4. Le modèle de Madingley (GEM)
Source : Harfoot et al. (2014), PLoS Biology 12:e1001841, [texte](https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.1001841). Code et documentation : [MadingleyModel.pdf](https://projects.nceas.ucsb.edu/nceas/attachments/download/1904/MadingleyModel.pdf). Code ouvert [S].

### 4.1 Périmètre
- Masses de 10 µg à 150 000 kg [S].
- Cellules de 1°×1° (focales) ou 2°×2° (globales). Latitudes > 65° exclues. Océan : 100 m supérieurs [S].
- Pas de temps : le mois (runs principaux) ; le jour a été testé [S].
- Durées : 1 000 ans (cellules focales), 100 ans (global). 10 réplicats par condition [S].

### 4.2 Agrégation en cohortes
- Autotrophes : stocks de biomasse, pas d'individus [S].
- Hétérotrophes : cohortes. Une cohorte regroupe les individus d'une même cellule, d'un même groupe fonctionnel, avec des traits continus (masse de naissance, masse adulte) [S].
- État de chaque cohorte : abondance, masse individuelle, masse de reproduction stockée [S].
- Groupes fonctionnels : catégoriels (mode d'alimentation, milieu, endo- ou ectothermie). 9 terrestres, 10 marins [S].
- Initialisation : environ 112 cohortes par groupe terrestre, soit environ 1 000 cohortes par cellule [S].
- Fusion : quand le nombre dépasse un seuil (testé entre 500 et 10 000), on fusionne les paires les plus proches dans l'espace des traits, dans le même groupe [S].
- Scission : pas par défaut [S].
- Dispersion : la cohorte entière se déplace [S].

### 4.3 Équations (en mots)
- **Croissance** : Δm = assimilation (herbivorie + prédation) − métabolisme − allocation à la reproduction [S].
- **Prédation** : Holling III. Taux d'attaque de l'herbivore ∝ sa masse. Probabilité proie-prédateur gaussienne autour d'une taille optimale [S].
- **Temps de manipulation** : carnivores et omnivores = linéaire en masse de proie, en puissance décroissante de la masse du prédateur. Herbivores = puissance décroissante de la masse de l'herbivore [S].
- **Métabolisme** : loi de puissance (Brown), paramètres issus de taux métaboliques de terrain [S].
- **Reproduction** : à maturité, l'excédent est stocké. Un événement a lieu quand la réserve atteint une proportion seuil de la masse adulte [S].
- **Mortalité** : fond constant, famine (perte de masse par rapport au maximum atteint), sénescence (croissance exponentielle de type Gompertz après maturité), prédation (somme sur les cohortes prédatrices). Dans une cohorte, l'abondance ne fait que décroître [S].
- **Dispersion** : juvéniles par diffusion aléatoire ; dispersion active en cas de famine ; en mer, advection-diffusion par courants [S].
- **Non lus** (absents de l'extrait) : valeurs numériques des taux d'attaque, exposants de manipulation, efficacités d'assimilation, seuil de reproduction, mortalité de fond, taux de dispersion. Ils sont dans les tables 5 et 6 et le Text S1 du papier, à lire avant calibrage.

### 4.4 Ce qu'on garde, ce qu'on change
| Madingley | Tellus | Raison |
|---|---|---|
| Groupes fonctionnels fixes | Traits continus, lignées émergentes | Les espèces doivent apparaître |
| Pas d'évolution | Mutation, sélection, dérive, spéciation | Exigence du projet |
| ~1 000 cohortes/cellule, au mois | ≤ 32 cohortes/cellule, à l'année, échelonné | Budget CPU (6 cœurs, 8 ms/tick) |
| Holling III | III pour prédateurs, II pour herbivores | Stabilité, choix explicite |
| Grille 1° | 40 000–160 000 cellules (côté ≈ 110 à 55 km) | Échelle du projet |

Coût comparé : 160 000 cellules × 12 pas × 1 000 cohortes ≈ 1,9 × 10⁹ mises à jour par année simulée. Tellus : 16 000 cellules par tick (1/10) × 8 cohortes en moyenne ≈ 1,3 × 10⁵ par an. Écart d'environ 15 000, soit quatre ordres de grandeur.

## 5. Génome numérique
### 5.1 Représentation
Pas de séquence de nucléotides. Chaque lignée porte un vecteur de traits moyen μ (NT valeurs) et une variance σ² par trait : c'est le modèle des moments (Lande 1976) [M].

Chaque cohorte porte un effectif et une copie décalée de μ ; la variance est au niveau de la lignée, ce qui limite la mémoire. Traits normalisés dans [0, 1], avec traduction vers des unités physiques.

### 5.2 Liste de traits (NT ≈ 18) [D]
| # | Trait | Unité physique | Compromis principal |
|---|---|---|---|
| 1 | log10 masse adulte | µg → kg | Allométrie : le reste en découle |
| 2 | Rapport masse naissance / adulte | sans | Nombre contre qualité des petits |
| 3 | Régime (herbivore → carnivore) | sans | Niche trophique |
| 4 | T optimum | °C | Adaptation thermique |
| 5 | Largeur de tolérance thermique | °C | Spécialiste contre généraliste |
| 6 | Besoin hydrique (P min) | mm/an | Sec contre humide |
| 7 | Mobilité (dispersion) | km/an | Dispersion contre entretien |
| 8 | Fraction d'énergie vers la reproduction | sans | r contre K |
| 9 | Longévité / sénescence | sans | Fécondité tardive contre jeunesse |
| 10 | Recombinaison (mode de reproduction) | sans | Vitesse d'adaptation contre coût |
| 11 | Milieu aquatique ↔ terrestre | sans | Branchies contre poumons |
| 12 | Défense (armure, toxines) | sans | Défense contre croissance |
| 13 | Efficacité de chasse | sans | Prédation contre coût |
| 14 | Grégarisme | sans | Effet de groupe contre compétition locale |
| 15 | Tolérance au feu | sans | Contre croissance |
| 16 | Taux de croissance intrinsèque | j⁻¹ | Contre entretien |
| 17 | Photosynthèse | sans | 0 au départ (transition 7.2) |
| 18 | Adhérence cellulaire | sans | 0 au départ (transition 7.4) |

Les traits 17 et 18 valent 0 au départ. Ils apparaissent par mutation, à faible probabilité, sans script [D].

### 5.3 Compromis
- Masse ↔ métabolisme : imposé par la MTE, pas un choix [S].
- Mobilité ↔ fécondité : `f_eff = f · (1 − κ · mobilité)` [D].
- Largeur thermique ↔ pic de performance : `perf(T) = P_0 · (w_0 / w)^α · exp(−(T − T_opt)² / (2w²))` [D]. Le généraliste a un pic plus bas.
- Défense ↔ croissance : `r_eff = r · (1 − δ · défense)` [D].
- Colonisation ↔ compétition : modèle de Tilman (1.4).
- Adhérence ↔ dispersion : [D].

Tous les coûts entrent dans b, m et f ; aucun n'est un bonus.

### 5.4 Mutation
À chaque naissance, chaque trait reçoit une perturbation gaussienne d'écart-type σ_m · √μ_mut, avec μ_mut le taux de mutation par génération (≈ 10⁻³ à 10⁻² par trait [M]). Bornes [0, 1] par réflexion, pas par troncature [D].

Les mutations délétères coûtent via les traits ; la sélection fait le tri. Aucune mutation n'est « ajustée ».

### 5.5 Sélection
Pas de fitness codée. La fitness est la croissance locale de la lignée : r = b − m à la densité présente. Les lignées à r plus faible sont exclues par compétition [D]. C'est une sélection réelle, issue de la dynamique des populations.

### 5.6 Dérive
À petit N, naissances et morts sont tirées par échantillonnage binomial (type Wright-Fisher), avec un RNG graîné par (graine, tick, cellule, lignée) [D]. La dérive est nécessaire à la spéciation allopatrique.

Perte de variance par consanguinité : 1/(2N_e) par génération [M]. À implémenter avec un N_e par cohorte.

## 6. Spéciation et extinction
### 6.1 Définitions
- Espèce biologique : groupes reproductivement isolés (Mayr [M]). Simulé par absence de flux génique.
- Isolement allopatrique : absence d'échange entre sous-populations depuis T_iso années [D].
- Distance génétique d_gen : norme pondérée par σ sur les traits de compatibilité (traits 10 et 14, plus un sous-trait de compatibilité) [D].
- Incompatibilité de Dobzhansky-Muller : allèles délétères en combinaison, sur des loci différents, apparus dans des populations isolées. C'est la cause dominante de l'isolement postzygotique [S, Turelli et al. 2014](https://pmc.ncbi.nlm.nih.gov/articles/PMC4142334). Dans Tellus : chaque paire de lignées qui s'écartent accumule des incompatibilités, qui réduisent la fitness des hybrides au retour du contact.

### 6.2 Règle de scission (allopatrique) [D]
Une lignée L se scinde si :
1. deux sous-populations de L sont isolées depuis au moins T_iso (≈ 500 à 2 000 ans [D]) ;
2. d_gen(A, B) > θ, avec θ calibré pour produire 10 à 100 scissions par 10 000 ans sur 160 000 cellules [D] ;
3. au retour du contact, les hybrides ont une fitness réduite d'un facteur (1 − incompatibilité).

Le seuil d'isolement reproductif est de l'ordre d'une distance de Nei D ≈ 0,5, selon Coyne et Orr (2004, p. 75), cité par Turelli et al. [S, chiffre à vérifier dans le livre].

### 6.3 Spéciation sympatrique (rare, optionnelle)
Si la distribution des traits d'une cellule devient bimodale sous sélection disruptive, une lignée peut se scinder sans barrière géographique (Dieckmann et Doebeli 1999 [M]). Probabilité annuelle proportionnelle à la largeur de la bimodalité et à l'intensité de la sélection disruptive. Désactivée au début, pour rester conservateur [D].

### 6.4 Phylogénie
Une scission = un enregistrement (id_parent, id_fils, tick, μ, cellule). Quelques milliers par 10 000 ans, négligeable.

### 6.5 Extinction
- Extinction de lignée : N_total < N_min (défaut : 10 individus, ou 0 cellule occupée) pendant T_ext = 50 ans [D].
- Extinction de fond : taux fossile de l'ordre de 0,1 à 1 espèce par million d'espèces et par an [M]. Pour quelques milliers de lignées, cela donne environ une extinction par décennie à quelques par millénaire (test 5).
- Extinctions de masse : pas de déclencheur codé. Elles émergent de chocs injectés par `god/` ou par le climat (volcan, météorite, refroidissement, baisse de NPP) [D]. Une extinction de masse se mesure : biomasse −50 % ou lignées −30 % en moins de 500 ans [D].

## 7. Grandes transitions
Principe [D] : chaque transition est un état. Il s'atteint quand des conditions sont réunies (température, énergie, chimie, O₂, lumière), avec une probabilité par cellule et par an. Pas de date. Un état est un trait ou une concentration chimique qui apparaît, pas un objet.

Référence : Szathmáry et Maynard Smith (1995, Nature 374:227, doi:10.1038/374227a0) [S] : huit transitions dans la transmission de l'information. Szathmáry (2015) a retiré le sexe de la liste [S].

### 7.0 Problème d'échelle temporelle [D, décision Chris]
Le projet vise 10 000 ans simulés. Sur Terre, la vie apparaît il y a 3,5 à 4 milliards d'années [M]. Avec un tick annuel, cela ferait 4 × 10⁹ ticks : hors budget (8 ms par tick).

Trois options :
- (a) ères géologiques en ticks de 10³ à 10⁴ ans avant l'ère vivante, avec un Δt variable par époque. Les probabilités deviennent p = 1 − exp(−λ·Δt). Les gardes de sous-pas s'adaptent.
- (b) départ à une époque où la chimie prébiotique est en place. L'abiogenèse reste probabiliste, mais le point de départ est calibré.
- (c) abiogenèse hors simulation, par preset de graine. Cela renonce à l'émergence.

Recommandation [D] : (a), car elle garde l'exigence « rien n'est préfait ». À confirmer avec Chris.

### 7.1 Abiogenèse
- Modèle [M, Hordijk et Steel 2004, J. Theor. Biol.] : un ensemble autocatalytique autoentretenu (RAF) apparaît plus souvent quand augmentent le nombre d'espèces chimiques, la diversité des réactions et l'énergie disponible.
- Règle [D] : p_abio(cellule, an) = λ_0 · f_énergie · f_diversité · f_surface, avec chaque facteur dans [0, 1]. Énergie : hydrothermal, foudre, UV. Diversité : composés présents. Surface : minéraux et argiles [M].
- Une protocellule devient une lignée à N initial. Elle disparaît si N < N_min.

### 7.2 Photosynthèse
- Trait 17 à 0. Apparition par mutation (μ_photo ≈ 10⁻⁶ par naissance [D]) dans une lignée limitée par l'énergie chimique.
- Avantage physique : gain proportionnel à la lumière reçue × surface de capture. La lignée prend le dessus quand l'énergie chimique se raréfie.
- O₂ [M] : production nette moins puits (volcanisme, roches). C'est un stock global de l'atmosphère, pas un état de cellule.

### 7.3 Eucaryotes
- Lane et Martin (2010, Nature 467:929, doi:10.1038/nature09486) : la complexité eucaryote s'explique par l'énergie disponible par gène. L'argument est débattu [S, Lynch et Marinov 2015 via recherche]. Le facteur 200 000 de la presse est contesté ; un article PMC donne environ × 2 000 [S].
- Règle [D] : deux lignées procaryotes co-localisées, dont l'une respire en aérobie, avec O₂ au-dessus d'un seuil. p_endo = λ · co-localisation · f_O₂. Résultat : une lignée avec un trait « taille de génome » et une capacité énergétique par cellule plus grande.
- Ne coder aucun chiffre issu du débat, seulement le mécanisme.

### 7.4 Multicellularité
- Ratcliff et al. (2012, PNAS) [S, titre et date à confirmer] : sélection par sédimentation sur levure. Des « flocons » apparaissent en environ 60 jours [S, via presse].
- Ratcliff et al. (2015, Nature Communications 6:6102, doi:10.1038/ncomms7102) [S] : le phénotype vient de l'inhibition du facteur ACE2, avec des règles de croissance déterministes et une héritabilité élevée.
- Règle [D] : trait 18 (adhérence). Une lignée adhérente, mangée par des prédateurs à petite bouche qui ne peuvent avaler un amas, gagne en fitness. Un amas stable devient une lignée multicellulaire, avec un trait « différenciation » ajouté à 0.

### 7.5 Sortie des eaux
- Conditions [M] : O₂ suffisant pour une couche d'ozone (filtre UV), ressources minérales terrestres, eau de surface, PFT adaptés.
- Règle [D] : une lignée au trait 11 (milieu) peut transitionner, avec une probabilité annuelle si la cellule est terrestre, voisine d'un océan, et si O₂ ≥ seuil. Le trait 6 (besoin hydrique) limite sa dispersion.

### 7.6 Synthèse
| Transition | Condition déclenchante | Probabilité | Trait ajouté |
|---|---|---|---|
| Abiogenèse | Énergie + diversité chimique + surface | λ₀ × facteurs | Lignée initiale |
| Photosynthèse | Lumière + réduits limitants | μ_photo / naissance | 17 |
| Eucaryote | Co-localisation + O₂ | λ × co-loc. × f_O₂ | Taille de génome |
| Multicellularité | Prédation + adhérence | μ_adh × sélection | 18 + différenciation |
| Sortie des eaux | O₂ + terre voisine d'un océan | λ × f_terre | 11 (milieu) |

## 8. Comportements au niveau cohorte
Principe [D] : un comportement est une règle de flux entre cellules, ou un coefficient sur un taux, dépendant des traits. Aucun comportement n'est un objet.

- **Fuite** : fraction f_fuite = clamp(pression_prédateur × vitesse, 0, f_max) de la cohorte quitte la cellule vers la voisine la moins prédatée. Coût : mobilité (trait 7).
- **Chasse en groupe** : taux d'attaque effectif = a · N_prédateurs^γ, avec γ ≈ 1,2 pour les grégaires et γ = 1 pour les solitaires [D]. Le trait 14 fait varier γ entre 1 et 1,3.
- **Migration saisonnière** : flux vers la voisine de meilleure NPP prévue au trimestre suivant. Fraction = trait migration × gradient de NPP. Coût : métabolisme × distance.
- **Territorialité** : densité maximale K_terr par cellule, dérivée de la taille de territoire, qui croît en M^(0,75–1) [M]. Une cohorte territoriale refuse la dispersion entrante au-delà de K_terr.
- **Mouvement collectif** (boids, Reynolds 1987 [M] ; Couzin et al. 2002 [M]) : seulement pour la vue inspecteur ou le niveau de détail élevé, jamais pour la simulation.

## 9. Artificial life et jeux
| Référence | Ce qu'on en retient | Ce qu'on ne copie pas |
|---|---|---|
| **Polyworld** (Yaeger 1994 [M]) | Énergie locale, agents à cerveau ; comportements et spéciation émergent | Agents individuels : trop coûteux |
| **Avida** (Ofria et Wilke 2004, Artificial Life 10:191, doi:10.1162/106454604773563612 [S]) | Récompenser des tâches simples fait apparaître des fonctions complexes par étapes. Cadre expérimental reproductible | Organismes individuels |
| **Tierra** (Ray 1991 [M]) | Parasites et hyperparasites émergent quand la mémoire est limitée | Code auto-réplicant |
| **Species: Artificial Life, Real Evolution** ([MobyGames](https://www.mobygames.com/game/121512)) | Génomes d'espèces, clades, lecture visuelle de la diversité. Le développeur dit que le climat local influence la spéciation [S]. Un avis de joueur dit qu'un généraliste dominant bloque la spéciation ultérieure [S, avis non vérifié] | Implémentation. Statut : early access, probablement abandonné [S] |
| **Thrive** ([wiki](https://wiki.revolutionarygamesstudio.com/wiki/What_is_Thrive)) | Cellule-organisme, dynamique de population avec mutations, éditeur de traits. Open source, Godot [S] | Échelle cellulaire |
| **Equilinox** (Karl Wimble, 2018, [GamingOnLinux](https://www.gamingonlinux.com/2019/01/equilinox-is-a-rather-beautiful-and-peaceful-nature-sim-available-on-linux/page=1/)) | Écosystème lisible. Ressources dépendant du confort des animaux. Évolution par le joueur. Un commentaire suppose que l'absence de propagation évite l'instabilité [S, avis non officiel] | Design de gameplay |

Leçon transversale : le généraliste dominant de Species est un problème de compétition sans structure spatiale. Tellus doit donc garder des barrières de dispersion, sinon la spéciation s'éteint.

## Règles pour Tellus
### Structures de données [D]
```rust
// simulation/src/life/evolution/lineage.rs
pub struct Lineage {
    pub id: u32,
    pub parent: Option<u32>,
    pub born_tick: u32,
    pub split_tick: Option<u32>,
    pub extinct_tick: Option<u32>,
    pub mu: [f32; NT],   // moyenne des traits, dans [0, 1]
    pub var: [f32; NT],  // variance intra-lignée (modèle des moments)
    pub n_total: f64,    // effectif global
    pub isolation: u32,  // ticks depuis le dernier échange génique
}

// simulation/src/life/fauna/cohort.rs
pub struct Cohort {
    pub lineage: u32,      // -> Lineage
    pub cell: u32,         // index de cellule (grille géodésique)
    pub n: f32,            // abondance
    pub mass_ind: f32,     // masse individuelle (kg)
    pub mass_repro: f32,   // réserve de reproduction (kg)
    pub mass_adult: f32,   // masse adulte (kg), dérivée du génome
    pub age: u16,          // ticks
}
```

Plafond : 32 cohortes par cellule ; au-delà, fusion (4.2). Mémoire : environ 48 o par cohorte, soit 160 000 × 8 × 48 o ≈ 61 Mo en moyenne. Les PFT et les nutriments par cellule vont dans des tableaux SoA à part.

Découpage : `life/` regroupe flore, faune, génome, évolution et transitions. Ces sous-dossiers devront être définis dans `ARCHITECTURE.md` pour rester non chevauchants.

### Pseudo-code par tick (1 tick = 1 an) [D]
```
tick_vivant(monde, t):
  tranche = cellules où (id + t) mod 10 == 0           // 1/10 des cellules
  pour c dans tranche:
     env = climat(c)                                    // T, P, insolation (moyennes annuelles)
     npp = min(miami_T(env.T), miami_P(env.P), azote(c), phosphore(c))
     flore.mise_a_jour(c, npp, feu)                     // Tilman + feu stochastique
  pour c dans tranche:
     pour k dans cohortes(c):
        met = B0 · M^0.75 · exp(-E/kT) · actif(k, T)    // sous-pas si r·Δt >= 1
        mange(k, proies_dans(c))                        // Holling II/III, gaussienne de taille
        reproduction(k) ; mortalite(k)                  // fond, famine, sénescence, prédation
     fusion_si_plus_de(32, c)
  dispersion(monde.graphe)                              // diffusion conservative + noyau mixte
  speciation(monde, t)                                  // isolement >= T_iso et d_gen > θ
  extinction(monde, t)                                  // N_total < N_min pendant T_ext
  transitions(monde, t)                                 // probabilité par cellule, aucun script
  mesures(monde, t)                                     // biomasse par niveau, lignées vivantes
```

Chaque fonction reste sous 35 lignes (`code-standards.md`). Le RNG est `shared::rng(graine, t, cellule)`. Les flux de C, N, P passent par un registre de comptabilité, pour que la conservation soit testable.

### Budget [D]
- 160 000 cellules, mise à jour de 1/10 par tick : 16 000 cellules par tick.
- 8 cohortes en moyenne (32 au maximum) : 1,3 × 10⁵ mises à jour de cohortes par tick, 5 × 10⁵ au pire.
- Coût supposé ≤ 30 ns par mise à jour : 4 ms de CPU, soit environ 0,7 ms de temps mur sur 6 cœurs. Dans les 8 ms de PROJET.md. À mesurer avec `pnpm bench`.
- Zone regardée en détail : sous-pas mensuel, 12 fois plus cher, sur quelques cellules seulement.

## Tests de validation
Seuils [D], à calibrer. Tests Rust sous `cargo test life::...`, benchmark sous `pnpm bench`.

1. **Populations bornées (10 000 ans)** : 5 graines × 3 presets. Aucune lignée au-delà de 10³ × K_local, aucun NaN, aucune croissance soutenue de plus de ×100 sur 50 ans. Critère de réussite de PROJET.md.
2. **Pyramide trophique** : biomasse(niveau n+1) / biomasse(niveau n) ∈ [0,01 ; 0,3], médiane ≈ 0,1. Biomasse végétale / biomasse animale > 10. Mesuré après 2 000 ans, à l'état quasi stationnaire.
3. **Efficacité trophique mesurée** : transfert effectif par niveau ∈ [5 %, 20 %], via le registre de flux.
4. **Lignées vivantes** : 10² à 10³ en régime établi (après 5 000 ans). Ordre de grandeur, pas cible : la biodiversité réelle est d'environ 8,7 millions d'espèces eucaryotes [M, Mora et al. 2011]. Turnover > 0 : au moins une scission ou extinction tous les 1 000 ans.
5. **Extinction de fond** : 1 à 50 extinctions de lignées par 10 000 ans sans choc. Au moins un événement de masse (biomasse −50 % ou lignées −30 % en moins de 500 ans) quand un choc est injecté via `god/`.
6. **Lotka-Volterra** : dérive de V < 1 % sur 1 000 ans simulés, Δt = 0,1 an, RK2. Test unitaire isolé.
7. **Kleiber** : pente log-log métabolisme / masse = 0,75 ± 0,02 sur 6 ordres de grandeur de masse, à température fixe.
8. **MTE** : Q10 entre 283 K et 293 K ∈ [2,2 ; 2,6] pour E = 0,65 eV. Pente densité / masse ∈ [−0,85 ; −0,65] (Damuth) [M].
9. **Fisher-KPP** : vitesse du front 1D ∈ [0,8 ; 1,2] × 2√(rD). À D égal, le noyau mixte doit accélérer le front par rapport au gaussien (Kot et al. 1996).
10. **Biomes** : après 1 000 ans à climat fixe, la classe de Whittaker de la végétation dominante égale celle du climat pour au moins 70 % des cellules terrestres.
11. **Conservation** : C, N, P conservés à 10⁻⁹ relatif par tick. Respiration, clamp et feu sont comptés comme des pertes.
12. **Déterminisme** : même graine, même hash d'état après 10 000 ans. 1 thread et 6 threads donnent le même hash (réductions ordonnées).
13. **Spéciation allopatrique** : deux continents isolés (deux blocs de cellules sans dispersion inter-blocs). Au moins une scission par continent dans 80 % des graines sur 10 000 ans. d_gen > θ à chaque scission. Hybrides réduits au contact.
14. **Performance** : `pnpm bench`. Tick vivant < 8 ms sur 160 000 cellules, 6 cœurs (PROJET.md, E3.S5).
15. **Abiogenèse** (si option 7.0 a) : sur 50 graines, premier état vivant dans [0,5 ; 2] × durée de référence, et jamais avant que l'énergie et la diversité chimique soient réunies.
16. **Aucun script** : garde de lint. Aucun `if tick ==` ni date codée dans `life/`.

## Sources
[S] Lues ou trouvées par recherche (URL ou DOI) : Harfoot et al. 2014, PLoS Biology, https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.1001841 ; documentation Madingley, https://projects.nceas.ucsb.edu/nceas/attachments/download/1904/MadingleyModel.pdf ; Brown et al. 2004, Ecology 85:1771 (non ouvert), https://people.clas.ufl.edu/gillooly/files/05_brownetal_2004.pdf ; Gillooly et al. 2001, Science 293:2248, https://pubmed.ncbi.nlm.nih.gov/11567137/ ; équations de Miami, https://www.alice.cnptia.embrapa.br/alice/bitstream/doc/852804/1/ID410252009CBAcd217326.pdf ; Sitch et al. 2003, GCB 9:161, https://impacts.ucar.edu/en/publications/evaluation-of-ecosystem-dynamics-plant-geography-and-terrestrial-/ ; Thonicke et al. 2010, doi:10.5194/bg-7-1991-2010 ; Kot et al. 1996, Ecology 77:2027, https://era.library.ualberta.ca/files/kw52j9322/Ecology_77_1996_2027.pdf ; Szathmáry et Maynard Smith 1995, doi:10.1038/374227a0 ; Lane et Martin 2010, doi:10.1038/nature09486 ; Ratcliff et al. 2015, doi:10.1038/ncomms7102 ; Ofria et Wilke 2004, doi:10.1162/106454604773563612 ; Turelli et al. 2014, https://pmc.ncbi.nlm.nih.gov/articles/PMC4142334 ; Welch 2004, doi:10.1554/03-502 ; Species, https://www.mobygames.com/game/121512 ; Thrive, https://wiki.revolutionarygamesstudio.com/wiki/What_is_Thrive ; Equilinox, https://www.gamingonlinux.com/2019/01/equilinox-is-a-rather-beautiful-and-peaceful-nature-sim-available-on-linux/page=1/

[M] Mémoire de référence, à confirmer : Whittaker 1975 ; Lieth 1975 ; Holling 1959 ; Lindeman 1942 ; Kleiber 1932 ; May 1972 ; Tilman 1994 ; Botkin et al. 1972 ; Damuth 1981 ; Lande 1976 ; Dieckmann et Doebeli 1999 ; Mayr ; Raup et Sepkoski 1982 ; Hordijk et Steel 2004, J. Theor. Biol. ; Yaeger 1994 ; Ray 1991 ; Reynolds 1987 ; Couzin et al. 2002 ; Mora et al. 2011, PLoS Biology ; Field et al. 1998, Science ; Coyne et Orr 2004, Speciation, Sinauer.
