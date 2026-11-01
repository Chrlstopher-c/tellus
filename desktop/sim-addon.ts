// Chargement et typage de l'addon natif de simulation (exports napi du pont v2, REGLES §13).
import { createRequire } from 'node:module';
import { join } from 'node:path';

export const ADDON_PATH = join(__dirname, '../native/index.node');

export interface SimAddon {
  createWorld(configJson: string): string;
  lookupIds(): Float32Array;
  lookupWeights(): Float32Array;
  schedule(wallSeconds: number, yearsPerSecond: number, lastTickSeconds: number): number;
  step(dtYears: number): number;
  snapshot(): Float32Array;
  vitals(): string;
  inspect(cell: number): string;
  intervene(json: string): string;
  drainEvents(): string;
}

const EXPORTS: readonly (keyof SimAddon)[] = [
  'createWorld', 'lookupIds', 'lookupWeights', 'schedule', 'step', 'snapshot', 'vitals', 'inspect', 'intervene',
  'drainEvents',
];

function isSimAddon(value: unknown): value is SimAddon {
  if (typeof value !== 'object' || value === null) return false;
  return EXPORTS.every((name) => typeof Reflect.get(value, name) === 'function');
}

export function loadAddon(): SimAddon {
  const requireAddon = createRequire(__filename);
  const loaded: unknown = requireAddon(ADDON_PATH);
  if (!isSimAddon(loaded)) throw new Error(`addon ${ADDON_PATH} sans exports attendus (${EXPORTS.join(', ')})`);
  return loaded;
}
