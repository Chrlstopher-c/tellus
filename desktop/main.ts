import { app, BrowserWindow, MessageChannelMain, screen, utilityProcess, type UtilityProcess } from 'electron';
import { join } from 'node:path';
import { SIM_PORT_CHANNEL } from '../interface/src/sim-bridge/protocol';
import { logger } from './logger';

const WINDOW_TITLE = 'Tellus';
const BACKGROUND_COLOR = '#04060a';
const RENDERER_HTML = join(__dirname, '../renderer/index.html');
const PRELOAD_SCRIPT = join(__dirname, '../preload/index.js');
const SIM_SCRIPT = join(__dirname, 'sim.js');
const SIM_CONNECT_MESSAGE = 'connect';

function forkSim(): UtilityProcess {
  const sim = utilityProcess.fork(SIM_SCRIPT, [], { serviceName: 'tellus-sim' });
  sim.on('exit', (code: number) => logger.warn({ code }, 'processus de simulation arrêté'));
  app.once('will-quit', () => sim.kill());
  return sim;
}

/** Relie la fenêtre au processus de simulation : un MessageChannel dont chaque extrémité va à un côté. */
function connectSim(win: BrowserWindow, sim: UtilityProcess): void {
  const { port1, port2 } = new MessageChannelMain();
  sim.postMessage(SIM_CONNECT_MESSAGE, [port1]);
  win.webContents.postMessage(SIM_PORT_CHANNEL, null, [port2]);
  logger.info('simulation reliée à la fenêtre');
}

function createMainWindow(): BrowserWindow {
  const { bounds } = screen.getPrimaryDisplay();
  const win = new BrowserWindow({
    x: bounds.x,
    y: bounds.y,
    width: bounds.width,
    height: bounds.height,
    title: WINDOW_TITLE,
    backgroundColor: BACKGROUND_COLOR,
    show: false,
    webPreferences: {
      preload: PRELOAD_SCRIPT,
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });
  win.once('ready-to-show', () => {
    win.setFullScreen(true);
    win.show();
    logger.info({ bounds }, 'fenêtre affichée');
  });
  return win;
}

async function loadRenderer(win: BrowserWindow): Promise<void> {
  const devUrl = process.env['ELECTRON_RENDERER_URL'];
  const source = devUrl ?? RENDERER_HTML;
  try {
    if (devUrl) await win.loadURL(devUrl);
    else await win.loadFile(RENDERER_HTML);
    logger.info({ source }, 'renderer chargé');
  } catch (error: unknown) {
    logger.error({ err: error, source }, 'échec du chargement du renderer');
  }
}

async function bootstrap(): Promise<void> {
  app.setName(WINDOW_TITLE);
  await app.whenReady();
  logger.info({ version: app.getVersion(), electron: process.versions.electron }, 'démarrage');
  const sim = forkSim();
  const win = createMainWindow();
  await loadRenderer(win);
  connectSim(win, sim);
}

app.on('window-all-closed', () => {
  app.quit();
});

bootstrap().catch((error: unknown) => {
  logger.error({ err: error }, 'démarrage impossible');
  app.quit();
});
