import { createRequire } from 'node:module';
import { join } from 'node:path';
import type { MessagePortMain } from 'electron';
import pino from 'pino';
import pretty from 'pino-pretty';
import type { SimCommand, SimReply, SimResult } from '../interface/src/sim-bridge/protocol';

const ADDON_PATH = join(__dirname, '../native/index.node');
const LOG_FILE = join(__dirname, '../../logs/sim.log');
const LEVEL = process.env['TELLUS_LOG_LEVEL'] ?? 'info';

const logger = pino(
  { name: 'tellus-sim', level: LEVEL },
  pino.multistream([
    { level: LEVEL, stream: pretty({ colorize: true }) },
    { level: LEVEL, stream: pino.destination({ dest: LOG_FILE, append: false, mkdir: true, sync: true }) },
  ]),
);

interface SimAddon {
  createWorld(seed: number, level: number): number;
  step(dtYears: number): number;
  snapshotElevation(): Float32Array;
}

function isSimAddon(value: unknown): value is SimAddon {
  if (typeof value !== 'object' || value === null) return false;
  return ['createWorld', 'step', 'snapshotElevation'].every(
    (name) => name in value && typeof Reflect.get(value, name) === 'function',
  );
}

function loadAddon(): SimAddon {
  const requireAddon = createRequire(__filename);
  const loaded: unknown = requireAddon(ADDON_PATH);
  if (!isSimAddon(loaded)) throw new Error(`addon ${ADDON_PATH} sans exports attendus`);
  return loaded;
}

function parseCommand(data: unknown): SimCommand | null {
  if (typeof data !== 'object' || data === null) return null;
  const kind = Reflect.get(data, 'type');
  const id = Reflect.get(data, 'id');
  if (typeof id !== 'number') return null;
  if (kind === 'snapshot') return { type: 'snapshot', id };
  if (kind === 'create') return parseCreate(data, id);
  if (kind === 'step') return parseStep(data, id);
  return null;
}

function parseCreate(data: object, id: number): SimCommand | null {
  const seed = Reflect.get(data, 'seed');
  const level = Reflect.get(data, 'level');
  return typeof seed === 'number' && typeof level === 'number' ? { type: 'create', id, seed, level } : null;
}

function parseStep(data: object, id: number): SimCommand | null {
  const dtYears = Reflect.get(data, 'dtYears');
  return typeof dtYears === 'number' ? { type: 'step', id, dtYears } : null;
}

function runCommand(addon: SimAddon, command: SimCommand): SimResult {
  switch (command.type) {
    case 'create':
      return { type: 'create', cells: addon.createWorld(command.seed, command.level) };
    case 'step':
      return { type: 'step', time: addon.step(command.dtYears) };
    case 'snapshot':
      return { type: 'snapshot', values: addon.snapshotElevation() };
  }
}

function toReply(addon: SimAddon, command: SimCommand): SimReply {
  try {
    return { id: command.id, ok: true, result: runCommand(addon, command) };
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    logger.error({ err: error, command: command.type }, 'commande de simulation échouée');
    return { id: command.id, ok: false, error: message };
  }
}

function handleMessage(addon: SimAddon, port: MessagePortMain, data: unknown): void {
  const command = parseCommand(data);
  if (!command) {
    logger.warn({ data }, 'commande de simulation invalide ignorée');
    return;
  }
  try {
    // Electron ne transfère que des ports : l'instantané part en copie (structured clone), pas en transfert.
    port.postMessage(toReply(addon, command));
  } catch (error: unknown) {
    logger.error({ err: error, command: command.type }, 'envoi de la réponse impossible');
  }
}

function attachPort(addon: SimAddon, port: MessagePortMain): void {
  port.on('message', (messageEvent) => handleMessage(addon, port, messageEvent.data));
  port.start();
  logger.info('fenêtre connectée à la simulation');
}

function bootstrap(): void {
  let addon: SimAddon;
  try {
    addon = loadAddon();
  } catch (error: unknown) {
    logger.fatal({ err: error, path: ADDON_PATH }, 'addon de simulation introuvable ou invalide');
    process.exit(1);
  }
  process.parentPort.on('message', (messageEvent) => {
    const [port] = messageEvent.ports;
    if (!port) {
      logger.warn('connexion sans port ignorée');
      return;
    }
    attachPort(addon, port);
  });
  logger.info({ path: ADDON_PATH }, 'processus de simulation prêt');
}

bootstrap();
