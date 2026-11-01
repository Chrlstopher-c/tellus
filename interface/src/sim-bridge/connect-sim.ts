import { rendererLogger } from '../shared/logger';
import { SIM_PORT_CHANNEL } from './protocol';
import { createSimClient, type SimClient } from './sim-client';

/** API exposée par desktop/preload.ts via contextBridge. */
export interface TellusSimApi {
  requestPort(): void;
}

declare global {
  interface Window {
    readonly tellusSim?: TellusSimApi;
  }
}

let clientPromise: Promise<SimClient> | null = null;

/** Un seul client par fenêtre : le port reçu du main ne se partage pas entre deux clients. */
export function getSimClient(): Promise<SimClient> {
  clientPromise ??= openClient();
  return clientPromise;
}

async function openClient(): Promise<SimClient> {
  const api = window.tellusSim;
  if (!api) throw new Error('API de simulation absente : preload non chargé');
  try {
    return createSimClient(await receivePort(api));
  } catch (error: unknown) {
    rendererLogger.error({ err: error }, 'connexion à la simulation impossible');
    throw error;
  }
}

/** Écoute avant de demander : le preload répond par window.postMessage, donc le listener doit exister d'abord. */
function receivePort(api: TellusSimApi): Promise<MessagePort> {
  return new Promise<MessagePort>((resolve, reject) => {
    const onMessage = (event: MessageEvent): void => {
      if (event.source !== window || event.data !== SIM_PORT_CHANNEL) return;
      window.removeEventListener('message', onMessage);
      const [port] = event.ports;
      if (port) resolve(port);
      else reject(new Error('message de simulation sans port'));
    };
    window.addEventListener('message', onMessage);
    api.requestPort();
  });
}
