import { rendererLogger } from '../shared/logger';
import type { SimCommand, SimReply, SimResult } from './protocol';

/** Sous-ensemble de MessagePort utilisé par le client (permet un port simulé en test). */
export interface SimPort {
  postMessage(message: SimCommand): void;
  onmessage: ((event: MessageEvent<SimReply>) => void) | null;
  close(): void;
}

export interface SimClient {
  createWorld(seed: number, level: number): Promise<number>;
  step(dtYears: number): Promise<number>;
  snapshotElevation(): Promise<Float32Array>;
  dispose(): void;
}

type DistributiveOmit<T, K extends PropertyKey> = T extends unknown ? Omit<T, K> : never;
type CommandBody = DistributiveOmit<SimCommand, 'id'>;

interface Pending {
  readonly resolve: (result: SimResult) => void;
  readonly reject: (error: Error) => void;
}

interface Channel {
  request(body: CommandBody): Promise<SimResult>;
  close(): void;
}

export function createSimClient(port: SimPort): SimClient {
  const channel = createChannel(port);
  return {
    createWorld: async (seed: number, level: number): Promise<number> =>
      cellsOf(await channel.request({ type: 'create', seed, level })),
    step: async (dtYears: number): Promise<number> => timeOf(await channel.request({ type: 'step', dtYears })),
    snapshotElevation: async (): Promise<Float32Array> => valuesOf(await channel.request({ type: 'snapshot' })),
    dispose: (): void => channel.close(),
  };
}

function createChannel(port: SimPort): Channel {
  const pending = new Map<number, Pending>();
  let nextId = 1;
  port.onmessage = (event: MessageEvent<SimReply>): void => settle(pending, event.data);
  return {
    request(body: CommandBody): Promise<SimResult> {
      const id = nextId;
      nextId += 1;
      return new Promise<SimResult>((resolve, reject) => {
        pending.set(id, { resolve, reject });
        try {
          port.postMessage({ ...body, id });
        } catch (error: unknown) {
          pending.delete(id);
          rendererLogger.error({ err: error, command: body.type }, 'envoi de commande à la simulation impossible');
          reject(toError(error));
        }
      });
    },
    close(): void {
      port.close();
      rejectAll(pending, 'client de simulation fermé');
    },
  };
}

function settle(pending: Map<number, Pending>, reply: SimReply): void {
  const entry = pending.get(reply.id);
  if (!entry) {
    rendererLogger.warn({ id: reply.id }, 'réponse de simulation sans requête en attente');
    return;
  }
  pending.delete(reply.id);
  if (reply.ok) entry.resolve(reply.result);
  else entry.reject(new Error(reply.error));
}

function rejectAll(pending: Map<number, Pending>, reason: string): void {
  for (const entry of pending.values()) entry.reject(new Error(reason));
  pending.clear();
}

function cellsOf(result: SimResult): number {
  if (result.type !== 'create') throw unexpectedResult(result, 'create');
  return result.cells;
}

function timeOf(result: SimResult): number {
  if (result.type !== 'step') throw unexpectedResult(result, 'step');
  return result.time;
}

function valuesOf(result: SimResult): Float32Array {
  if (result.type !== 'snapshot') throw unexpectedResult(result, 'snapshot');
  return result.values;
}

function unexpectedResult(result: SimResult, expected: SimResult['type']): Error {
  return new Error(`réponse de simulation « ${result.type} » reçue, « ${expected} » attendue`);
}

function toError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}
