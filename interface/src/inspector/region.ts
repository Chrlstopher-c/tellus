// Détail d'une région tel que la simulation le renvoie (commande `inspect`).
export type Role = 'producer' | 'herbivore' | 'predator';

export interface SpeciesLine {
  id: number;
  name: string;
  role: Role;
  population: number;
  trend: readonly number[];
}

export interface RegionDetail {
  lat: number;
  lon: number;
  elevation: number;
  temperature: number;
  precipitation: number;
  iceFraction: number;
  vegCover: number;
  nppGcM2Yr: number;
  soil: { water: number; carbon: number; nitrogen: number; phosphorus: number };
  biome: string | null;
  speciesCount: number;
  species: readonly SpeciesLine[];
}

/** Nom de surface quand le vivant n'a pas (encore) de biome ici. Purement descriptif, dérivé de l'état physique. */
export function surfaceName(r: RegionDetail): [string, string] {
  if (r.elevation < 0) {
    if (r.iceFraction > 0.5) return ['Banquise', 'Océan gelé en surface'];
    return r.elevation > -250 ? ['Plateau continental', 'Mer peu profonde'] : ['Océan ouvert', 'Plaine abyssale'];
  }
  if (r.iceFraction > 0.5) return ['Inlandsis', 'Glace permanente'];
  if (r.biome && r.vegCover > 0.02) {
    return [r.biome, r.vegCover > 0.5 ? 'Couvert végétal dense' : 'Couvert végétal clairsemé'];
  }
  return r.precipitation < 250 ? ['Désert minéral', 'Roche et sable, sans vie'] : ['Roche nue', 'Sols minéraux, sans vie'];
}

export const ROLE_LABEL: Record<Role, string> = { producer: 'Producteur', herbivore: 'Herbivore', predator: 'Prédateur' };
export const ROLE_COLOR: Record<Role, string> = { producer: 'var(--life)', herbivore: 'var(--time)', predator: 'var(--god)' };
