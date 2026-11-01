/**
 * Contrat entre le processus utilityProcess de simulation (desktop/sim-process.ts) et les fenêtres.
 * Fichier sans dépendance DOM ni Node, importable des deux côtés.
 */

export const SIM_PORT_CHANNEL = 'tellus:sim-port';

export type SimCommand =
  | { readonly type: 'create'; readonly id: number; readonly seed: number; readonly level: number }
  | { readonly type: 'step'; readonly id: number; readonly dtYears: number }
  | { readonly type: 'snapshot'; readonly id: number };

export type SimResult =
  | { readonly type: 'create'; readonly cells: number }
  | { readonly type: 'step'; readonly time: number }
  | { readonly type: 'snapshot'; readonly values: Float32Array };

export type SimReply =
  | { readonly id: number; readonly ok: true; readonly result: SimResult }
  | { readonly id: number; readonly ok: false; readonly error: string };
