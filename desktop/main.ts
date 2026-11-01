import { app, BrowserWindow, screen } from 'electron';
import { join } from 'node:path';
import { logger } from './logger';

const WINDOW_TITLE = 'Tellus';
const BACKGROUND_COLOR = '#04060a';
const RENDERER_HTML = join(__dirname, '../renderer/index.html');
const PRELOAD_SCRIPT = join(__dirname, '../preload/index.js');

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
  const win = createMainWindow();
  await loadRenderer(win);
}

app.on('window-all-closed', () => {
  app.quit();
});

bootstrap().catch((error: unknown) => {
  logger.error({ err: error }, 'démarrage impossible');
  app.quit();
});
