// Client de simulation côté fenêtre : requêtes à réponse + abonnement aux trames et événements poussés.
import { rendererLogger } from '../shared/logger';
import type {
  ChronicleEvent, CreateRequest, SimCommand, SimFrame, SimMessage, SimResult, SpeedId, WorldInfo,
} from './protocol';

/** Sous-ensemble de MessagePort utilisé par le client (permet un port simulé en test). */
export interface SimPort {
  postMessage(message: SimCommand): void;
  onmessage: ((event: MessageEvent<SimMessage>) => void) | null;
  close(): void;
}

export interface SimClient {
  createWorld(request: CreateRequest): Promise<WorldInfo>;
  control(playing: boolean, speed: SpeedId): Promise<void>;
  inspect(cell: number): Promise<string>;
  intervene(power: string, cell: number, magnitude: number): Promise<string>;
  onFrame(listener: (frame: SimFrame) => void): () => void;
  onEvents(listener: (events: readonly ChronicleEvent[]) => void): () => void;
  dispose(): void;
}

type DistributiveOmit<T, K extends PropertyKey> = T extends unknown ? Omit<T, K> : never;
type CommandBody = DistributiveOmit<SimCommand, 'id'>;

interface Pending {
  readonly resolve: (result: SimResult) => void;
  readonly reject: (error: Error) => void;
}

interface Hub {
  readonly pending: Map<number, Pending>;
  readonly frames: Set<(frame: SimFrame) => void>;
  readonly events: Set<(events: readonly ChronicleEvent[]) => void>;
}

export function createSimClient(port: SimPort): SimClient {
  const hub: Hub = { pending: new Map(), frames: new Set(), events: new Set() };
  port.onmessage = (event: MessageEvent<SimMessage>): void => dispatch(hub, event.data);
  const request = makeRequester(port, hub);
  const subscribe = <T>(set: Set<T>, listener: T): (() => void) => {
    set.add(listener);
    return () => set.delete(listener);
  };
  return {
    createWorld: async (r) => expect(await request({ type: 'create', request: r }), 'create').world,
    control: async (playing, speed) => { expect(await request({ type: 'control', playing, speed }), 'control'); },
    inspect: async (cell) => expect(await request({ type: 'inspect', cell }), 'inspect').json,
    intervene: async (power, cell, magnitude) =>
      expect(await request({ type: 'intervene', power, cell, magnitude }), 'intervene').json,
    onFrame: (listener) => subscribe(hub.frames, listener),
    onEvents: (listener) => subscribe(hub.events, listener),
    dispose: () => {
      port.close();
      for (const p of hub.pending.values()) p.reject(new Error('client de simulation fermé'));
      hub.pending.clear();
    },
  };
}

function makeRequester(port: SimPort, hub: Hub): (body: CommandBody) => Promise<SimResult> {
  let nextId = 1;
  return (body) => new Promise<SimResult>((resolve, reject) => {
    const id = nextId++;
    hub.pending.set(id, { resolve, reject });
    try {
      port.postMessage({ ...body, id });
    } catch (error: unknown) {
      hub.pending.delete(id);
      rendererLogger.error({ err: error, command: body.type }, 'envoi de commande à la simulation impossible');
      reject(error instanceof Error ? error : new Error(String(error)));
    }
  });
}

function dispatch(hub: Hub, message: SimMessage): void {
  if (message.kind === 'frame') {
    for (const listener of hub.frames) listener(message);
    return;
  }
  if (message.kind === 'events') {
    for (const listener of hub.events) listener(message.events);
    return;
  }
  const entry = hub.pending.get(message.id);
  if (!entry) {
    rendererLogger.warn({ id: message.id }, 'réponse de simulation sans requête en attente');
    return;
  }
  hub.pending.delete(message.id);
  if (message.ok) entry.resolve(message.result);
  else entry.reject(new Error(message.error));
}

function expect<K extends SimResult['type']>(result: SimResult, type: K): Extract<SimResult, { type: K }> {
  if (!isType(result, type)) throw new Error(`réponse « ${result.type} » reçue, « ${type} » attendue`);
  return result;
}

function isType<K extends SimResult['type']>(r: SimResult, type: K): r is Extract<SimResult, { type: K }> {
  return r.type === type;
}
