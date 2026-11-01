// Vitesses de simulation : miroir exact de `simulation/src/clock` (REGLES §1).
import type { SpeedId } from '../sim-bridge/protocol';

export type { SpeedId };

export interface Speed {
  id: SpeedId;
  label: string;
  description: string;
  yearsPerSecond: number;
  spin: number;
}

export const SPEEDS: readonly Speed[] = [
  { id: 'day', label: '1 j', description: 'un jour par seconde', yearsPerSecond: 1 / 365.25, spin: 0.22 },
  { id: 'year', label: '1 an', description: 'une année par seconde', yearsPerSecond: 1, spin: 0.05 },
  { id: 'century', label: '100 ans', description: 'un siècle par seconde', yearsPerSecond: 100, spin: 0.035 },
  { id: 'tenKyr', label: '10 ka', description: 'dix mille ans par seconde', yearsPerSecond: 1e4, spin: 0.03 },
  { id: 'myr', label: '1 Ma', description: 'un million d’années par seconde', yearsPerSecond: 1e6, spin: 0.03 },
  { id: 'tenMyr', label: '10 Ma', description: 'dix millions d’années par seconde', yearsPerSecond: 1e7, spin: 0.03 },
];

const NUMBER = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 });

/** Âge lisible : années entières jusqu'au million, puis Ma / Ga avec deux décimales. */
export function formatAge(years: number): { value: string; unit: string } {
  if (years < 1e6) return { value: NUMBER.format(Math.floor(years)), unit: 'ans' };
  if (years < 1e9) return { value: (years / 1e6).toLocaleString('fr-FR', { maximumFractionDigits: 2 }), unit: 'Ma' };
  return { value: (years / 1e9).toLocaleString('fr-FR', { minimumFractionDigits: 2, maximumFractionDigits: 3 }), unit: 'Ga' };
}
