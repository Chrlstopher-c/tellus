// Pouvoirs d'intervention (REGLES §8) : chacun devient une perturbation de l'état dans la simulation.
export type PowerId = 'meteor' | 'volcano' | 'climate' | 'species' | 'ice' | 'drought';

export interface Power {
  id: PowerId;
  label: string;
}

export const POWERS: readonly Power[] = [
  { id: 'meteor', label: 'Météorite' },
  { id: 'volcano', label: 'Éruption volcanique' },
  { id: 'climate', label: 'Forcer le climat' },
  { id: 'species', label: 'Introduire une espèce' },
  { id: 'ice', label: 'Glaciation' },
  { id: 'drought', label: 'Sécheresse' },
];
