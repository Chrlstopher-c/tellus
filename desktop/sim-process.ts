// Processus utilityProcess : possède le monde, fait tourner la boucle de ticks, sert les fenêtres (REGLES §13).
import { join } from 'node:path';
import type { MessagePortMain } from 'electron';
import pino from 'pino';
import pretty from 'pino-pretty';
import type { SimCommand, SimMessage, SimResult, SpeedId, Vitals, WorldInfo } from '../interface/src/sim-bridge/protocol';
import { parseCommand } from './sim-commands';
import { ADDON_PATH, loadAddon, type SimAddon } from './sim-addon';

const LOG_FILE = join(__dirname, '../../logs/sim.log');
const LEVEL = process.env['TELLUS_LOG_LEVEL'] ?? 'info';
const LOOP_MS = 100;
const YEARS_PER_SECOND: Record<SpeedId, number> = {
  day: 1 / 365.25, year: 1, century: 100, tenKyr: 1e4, myr: 1e6, tenMyr: 1e7,
};

const logger = pino(
  { name: 'tellus-sim', level: LEVEL },
  pino.multistream([
    { level: LEVEL, stream: pretty({ colorize: true }) },
    { level: LEVEL, stream: pino.destination({ dest: LOG_FILE, append: false, mkdir: true, sync: true }) },
  ]),
);

interface Loop {
  playing: boolean;
  speed: SpeedId;
  hasWorld: boolean;
  lastTickSeconds: number;
  lastWall: number;
  ports: Set<MessagePortMain>;
}

function broadcast(loop: Loop, message: SimMessage): void {
  for (const port of loop.ports) {
    try {
      port.postMessage(message);
    } catch (error: unknown) {
      logger.warn({ err: error }, 'fenêtre injoignable, port retiré');
      loop.ports.delete(port);
    }
  }
}

function createWorld(addon: SimAddon, loop: Loop, config: unknown, level: number): WorldInfo {
  const info: unknown = JSON.parse(addon.createWorld(JSON.stringify({ ...Object(config), level })));
  const num = (k: string): number => Number(Reflect.get(Object(info), k));
  loop.hasWorld = true;
  return {
    cells: num('cells'), seed: num('seed'), name: String(Reflect.get(Object(info), 'name')),
    lookup: { width: num('lookupWidth'), height: num('lookupHeight'), ids: addon.lookupIds(), weights: addon.lookupWeights() },
  };
}

function run(addon: SimAddon, loop: Loop, command: SimCommand): SimResult {
  switch (command.type) {
    case 'create':
      return { type: 'create', world: createWorld(addon, loop, command.request.config, command.request.level) };
    case 'control':
      loop.playing = command.playing;
      loop.speed = command.speed;
      return { type: 'control' };
    case 'inspect':
      return { type: 'inspect', json: addon.inspect(command.cell) };
    case 'intervene':
      return { type: 'intervene', json: addon.intervene(JSON.stringify(command)) };
  }
}

function handle(addon: SimAddon, loop: Loop, port: MessagePortMain, data: unknown): void {
  const command = parseCommand(data);
  if (!command) {
    logger.warn({ data }, 'commande de simulation invalide ignorée');
    return;
  }
  try {
    port.postMessage({ kind: 'reply', id: command.id, ok: true, result: run(addon, loop, command) } satisfies SimMessage);
  } catch (error: unknown) {
    logger.error({ err: error, command: command.type }, 'commande de simulation en échec');
    const message = error instanceof Error ? error.message : String(error);
    port.postMessage({ kind: 'reply', id: command.id, ok: false, error: message } satisfies SimMessage);
  }
}

function tick(addon: SimAddon, loop: Loop): void {
  const now = performance.now();
  const wall = (now - loop.lastWall) / 1000;
  loop.lastWall = now;
  if (!loop.hasWorld) return;
  const rate = loop.playing ? YEARS_PER_SECOND[loop.speed] : 0;
  const dt = rate > 0 ? addon.schedule(wall, rate, loop.lastTickSeconds) : 0;
  const t0 = performance.now();
  const simYears = dt > 0 ? addon.step(dt) : addon.step(0);
  loop.lastTickSeconds = (performance.now() - t0) / 1000;
  const effectiveRatio = rate > 0 && wall > 0 ? Math.min(1, dt / (rate * wall)) : 1;
  const vitals: unknown = JSON.parse(addon.vitals());
  broadcast(loop, { kind: 'frame', simYears, effectiveRatio, fields: addon.snapshot(), vitals: toVitals(vitals) });
  const events: unknown = JSON.parse(addon.drainEvents());
  if (Array.isArray(events) && events.length > 0) broadcast(loop, { kind: 'events', events });
}

function toVitals(v: unknown): Vitals {
  const n = (k: string): number => Number(Reflect.get(Object(v), k) ?? 0);
  return {
    o2Percent: n('o2Percent'), co2Ppm: n('co2Ppm'), meanTempC: n('meanTempC'), species: n('species'),
    biomassGtC: n('biomassGtC'), era: String(Reflect.get(Object(v), 'era') ?? ''),
  };
}

function bootstrap(): void {
  let addon: SimAddon;
  try {
    addon = loadAddon();
  } catch (error: unknown) {
    logger.fatal({ err: error, path: ADDON_PATH }, 'addon de simulation introuvable ou invalide');
    process.exit(1);
  }
  const loop: Loop = {
    playing: true, speed: 'century', hasWorld: false, lastTickSeconds: 0, lastWall: performance.now(), ports: new Set(),
  };
  process.parentPort.on('message', (messageEvent) => {
    const [port] = messageEvent.ports;
    if (!port) return void logger.warn('connexion sans port ignorée');
    port.on('message', (e) => handle(addon, loop, port, e.data));
    port.start();
    loop.ports.add(port);
    logger.info('fenêtre connectée à la simulation');
  });
  setInterval(() => {
    try {
      tick(addon, loop);
    } catch (error: unknown) {
      logger.error({ err: error }, 'tick de simulation en échec');
    }
  }, LOOP_MS);
  logger.info({ path: ADDON_PATH }, 'processus de simulation prêt');
}

bootstrap();
