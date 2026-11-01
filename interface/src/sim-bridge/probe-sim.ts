import { getSimClient } from './connect-sim';
import type { SimClient } from './sim-client';

export const SIM_SEED = 42;
export const SIM_LEVEL = 6;
const PROBE_DT_YEARS = 1;

export interface SimProbe {
  readonly cells: number;
  readonly time: number;
  readonly snapshotLength: number;
}

let probePromise: Promise<SimProbe> | null = null;

/** Sonde de bout en bout (monde L6, un pas, instantané) pour l'affichage de debug. Calculée une seule fois. */
export function getSimProbe(): Promise<SimProbe> {
  probePromise ??= getSimClient().then(runProbe);
  return probePromise;
}

async function runProbe(client: SimClient): Promise<SimProbe> {
  const cells = await client.createWorld(SIM_SEED, SIM_LEVEL);
  const time = await client.step(PROBE_DT_YEARS);
  const snapshot = await client.snapshotElevation();
  return { cells, time, snapshotLength: snapshot.length };
}
