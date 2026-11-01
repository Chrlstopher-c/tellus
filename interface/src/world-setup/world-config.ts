// Paramètres de création d'un monde, envoyés tels quels à la simulation (genèse ou corps réel).
export type WorldSource =
  | { kind: 'generated'; seed: number; waterPercent: number; sizePercent: number; distancePercent: number }
  | { kind: 'body'; name: string };

export interface WorldConfig {
  source: WorldSource;
  matureToYears: number | null;
}

/** Fiche de présentation des corps jouables (valeurs affichées, la simulation lit simulation/data/planets.json). */
export const BODIES: readonly { name: string; facts: string; color: string }[] = [
  { name: 'Mercure', facts: '0,39 UA · sans atmosphère · 167 °C', color: '#a39c94' },
  { name: 'Vénus', facts: '0,72 UA · 92 bar · 464 °C', color: '#d9b77a' },
  { name: 'Terre', facts: '1,00 UA · 1 bar · 15 °C', color: '#5b8fb9' },
  { name: 'Lune', facts: '1,00 UA · sans atmosphère · −20 °C', color: '#b9b5ae' },
  { name: 'Mars', facts: '1,52 UA · 6 mbar · −63 °C', color: '#c46f4a' },
];

export function worldTitle(c: WorldConfig, generatedName: string): { name: string; origin: string } {
  if (c.source.kind === 'body') return { name: c.source.name, origin: 'Système solaire · données NASA' };
  return { name: generatedName, origin: `Monde généré · graine ${c.source.seed.toLocaleString('fr-FR')}` };
}

const SYL = ['as', 'ca', 'lon', 've', 'yra', 'or', 'me', 'thal', 'is', 'ar', 'del', 'un', 'ka', 'sel', 'mi', 'ron'];

/** Nom déterministe tiré de la graine (deux ou trois syllabes). */
export function nameFromSeed(seed: number): string {
  let s = seed >>> 0;
  const next = (): number => (s = (Math.imul(s ^ (s >>> 15), 2246822519) + 3266489917) >>> 0);
  const n = 2 + (next() % 2);
  const word = Array.from({ length: n }, () => SYL[next() % SYL.length] ?? '').join('');
  return word.charAt(0).toUpperCase() + word.slice(1);
}
