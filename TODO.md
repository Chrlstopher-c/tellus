# TODO — Tellus

## En cours
- E1 — Squelette qui tourne (voir PROJET.md)

## Backlog (après MVP)
- V2 Civilisations : individus → groupes → normes → institutions → États, tout émergent.
- V3 Technologie : arbre ouvert généré par combinaison de principes (énergie, matériaux, information, biologie),
  sans plafond à 2026.
- Sauvegarde / chargement de mondes, partage d'une graine.
- Sourcer excentricité, obliquité et albédo de Titan et Europe dans planets.json (chargement en `MissingField`).
- Pont Rust : ajouter un logger (entrées non finies ignorées silencieusement dans `clock`).
- Genèse : érosion sans redépôt terrestre ni isostasie post-érosion ; 2 graines sur 10 hors fenêtre de profondeur moyenne (L6).
- Genèse : `tectonics_frozen_mvp` ne teste que le déterminisme (pas de temps dans l'API) — à revoir avec la dérive des plaques.
- Climat : préconditionneur multigrille (L7 à 470-650 ms par pas annuel, plafond 300 itérations atteint).
- Climat : seuil de déglaciation de la boule de neige ×1 000-10 000 CO₂ (littérature ~×350) — ajouter assombrissement de la glace par poussières/cendres.
