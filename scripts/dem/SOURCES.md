# Sources des MNT (corps réels)

Toutes les données sont publiques, produites par des agences américaines (NASA, NOAA, USGS) : domaine public.
Le téléchargement et l'empreinte SHA-256 sont dans `sources.py` ; `build.sh` les vérifie à chaque exécution.

| Corps | Source | Fichier | Résolution d'origine | SHA-256 | Référence d'altitude |
|---|---|---|---|---|---|
| Terre | NOAA NCEI, ETOPO5 — https://www.ngdc.noaa.gov/mgg/global/relief/ETOPO5/TOPO/ETOPO5/ETOPO5.DAT | `etopo5.DAT` (18,7 Mo) | 5' (12 px/°), entiers 16 bits gros-boutiste, ligne 0 = pôle Nord | `471d3dd5…93c3` | niveau de la mer |
| Mars | NASA PDS, MGS MOLA MEGDR, dataset MGS-M-MOLA-5-MEGDR-L3-V1.0 (DOI 10.17189/1519460) — https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/meg016/megt90n000eb.img | `mars_megt16.img` (33 Mo) | 16 px/°, médiane de la topographie | `d18d9b9a…e7b` | aréoïde |
| Vénus | NASA PDS, Magellan TOPO L2, MGN-V-RDRS-5-TOPO-L2-V1.0 (DOI 10.17189/1522513) — https://pds-geosciences.wustl.edu/mgn/mgn-v-rss-5-gravity-l2-v1/mg_5201/topo/topogrd.dat | `venus_topogrd.dat` (0,5 Mo) | 1° (360 × 180), km, réf. sphère 6051,848 km | `76e88aea…6010` | rayon moyen |
| Lune | NASA PDS, LRO LOLA LDEM_16 — https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/cylindrical/img/ldem_16.img | `moon_ldem16.img` (33 Mo) | 16 px/°, entiers 16 bits × 0,5 m | `a511e40d…54cb` | rayon moyen 1 737,4 km |
| Mercure | USGS, MESSENGER MDIS Global DEM 665 m v2 — https://planetarymaps.usgs.gov/mosaic/Mercury_Messenger_USGS_DEM_Global_665m_v2.tif | 531 Mo, non téléchargé en entier | 64 px/°, entiers × 0,5 m, NoData −32768 | non vérifié (lecture partielle) | rayon moyen 2 440 km |

## Traitement

- **Mercure** : le GeoTIFF complet (531 Mo) dépasse la limite de 50 Mo. Le script lit par requêtes HTTP partielles une ligne
  sur 16 et une colonne sur 16 (4 px/°, 720 requêtes d'environ 46 Ko). C'est une décimation par échantillonnage ponctuel,
  pas une moyenne : les détails sous 4 km sont perdus avant le rééchantillonnage sur la grille.
- **Vénus** : la grille globale à 4,6 km (Magellan GTDR, `mg_3002/gtdr`) pèse environ 64 Mo en tuiles de 2 Mo ; elle dépasse
  la limite de 50 Mo par modèle. Le modèle 1° du PDS est utilisé à la place. Longitude de la colonne 0 : 240° E
  (`AXIS_START` du label ; vérifié : pic de Maxwell Montes à 64,5° N, 4° E).
- **Terre** : l'ETOPO 2022 (60 s) et l'ETOPO2 (73 Mo zip) dépassent la limite ; ETOPO5 est retenu.
  Conséquence : la fosse des Mariannes est lissée, minimum L6 = −7 461 m (L7 = −8 353 m, cible « < −8 000 m » à L7 seulement).
- **Rééchantillonnage** : chaque cellule de la grille (`shared::grid`, L6 et L7) reçoit la moyenne de 64 sous-points
  répartis uniformément en surface dans sa calotte d'angle solide (interpolation bilinéaire sur le MNT). Centres et
  angles solides viennent de Rust (test ignoré `export_grid_centers`), pas d'un recalcul Python.
- **Format de sortie** : `simulation/data/dem/<corps>_L<n>.f32`, f32 little-endian, une altitude (m) par cellule.
