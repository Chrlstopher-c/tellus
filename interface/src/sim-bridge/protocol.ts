/**
 * Contrat entre le processus utilityProcess de simulation (desktop/sim-process.ts) et les fenêtres (REGLES §13).
 * Fichier sans dépendance DOM ni Node, importable des deux côtés.
 */

export const SIM_PORT_CHANNEL = 'tellus:sim-port';

export type SpeedId = 'day' | 'year' | 'century' | 'tenKyr' | 'myr' | 'tenMyr';

/** Table de rendu calculée par la simulation : 3 cellules (RGBA, A inutilisé) et 2 poids (RG) par texel. */
export interface CellLookup {
  readonly width: number;
  readonly height: number;
  readonly ids: Float32Array;
  readonly weights: Float32Array;
}

export interface CreateRequest {
  readonly config: unknown;
  readonly level: number;
}

export interface WorldInfo {
  readonly cells: number;
  readonly seed: number;
  readonly name: string;
  readonly lookup: CellLookup;
}

export interface Vitals {
  readonly o2Percent: number;
  readonly co2Ppm: number;
  readonly meanTempC: number;
  readonly species: number;
  readonly biomassGtC: number;
  readonly era: string;
}

export interface ChronicleEvent {
  readonly tick: number;
  readonly years: number;
  readonly kind: string;
  readonly title: string;
  readonly body: string;
  readonly cell: number | null;
}

export type SimCommand =
  | { readonly type: 'create'; readonly id: number; readonly request: CreateRequest }
  | { readonly type: 'control'; readonly id: number; readonly playing: boolean; readonly speed: SpeedId }
  | { readonly type: 'inspect'; readonly id: number; readonly cell: number }
  | { readonly type: 'intervene'; readonly id: number; readonly power: string; readonly cell: number;
    readonly magnitude: number };

export type SimResult =
  | { readonly type: 'create'; readonly world: WorldInfo }
  | { readonly type: 'control' }
  | { readonly type: 'inspect'; readonly json: string }
  | { readonly type: 'intervene'; readonly json: string };

export type SimReply =
  | { readonly kind: 'reply'; readonly id: number; readonly ok: true; readonly result: SimResult }
  | { readonly kind: 'reply'; readonly id: number; readonly ok: false; readonly error: string };

/** Trame poussée par la simulation, au plus 10 par seconde. */
export interface SimFrame {
  readonly kind: 'frame';
  readonly simYears: number;
  readonly effectiveRatio: number;
  readonly fields: Float32Array;
  readonly vitals: Vitals;
}

export interface SimEvents {
  readonly kind: 'events';
  readonly events: readonly ChronicleEvent[];
}

export type SimMessage = SimReply | SimFrame | SimEvents;
