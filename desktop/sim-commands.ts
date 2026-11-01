// Validation des commandes reçues des fenêtres (données non fiables jusqu'à preuve du contraire).
import type { SimCommand, SpeedId } from '../interface/src/sim-bridge/protocol';

const SPEEDS: readonly SpeedId[] = ['day', 'year', 'century', 'tenKyr', 'myr', 'tenMyr'];

const get = (o: object, k: string): unknown => Reflect.get(o, k);
const isNum = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v);
const isSpeed = (v: unknown): v is SpeedId => typeof v === 'string' && SPEEDS.some((s) => s === v);

function parseCreate(o: object, id: number): SimCommand | null {
  const request = get(o, 'request');
  if (typeof request !== 'object' || request === null) return null;
  const level = get(request, 'level');
  return isNum(level) ? { type: 'create', id, request: { config: get(request, 'config'), level } } : null;
}

function parseControl(o: object, id: number): SimCommand | null {
  const playing = get(o, 'playing'), speed = get(o, 'speed');
  return typeof playing === 'boolean' && isSpeed(speed) ? { type: 'control', id, playing, speed } : null;
}

function parseIntervene(o: object, id: number): SimCommand | null {
  const power = get(o, 'power'), cell = get(o, 'cell'), magnitude = get(o, 'magnitude');
  if (typeof power !== 'string' || !isNum(cell) || !isNum(magnitude)) return null;
  return { type: 'intervene', id, power, cell, magnitude };
}

export function parseCommand(data: unknown): SimCommand | null {
  if (typeof data !== 'object' || data === null) return null;
  const id = get(data, 'id');
  if (!isNum(id)) return null;
  switch (get(data, 'type')) {
    case 'create': return parseCreate(data, id);
    case 'control': return parseControl(data, id);
    case 'inspect': {
      const cell = get(data, 'cell');
      return isNum(cell) ? { type: 'inspect', id, cell } : null;
    }
    case 'intervene': return parseIntervene(data, id);
    default: return null;
  }
}
