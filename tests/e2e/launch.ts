import { _electron, type ElectronApplication } from '@playwright/test';
import { join } from 'node:path';

const ROOT = process.cwd();
const MAIN_ENTRY = join(ROOT, 'out/main/index.js');

export async function launchTellus(): Promise<ElectronApplication> {
  return _electron.launch({ args: [MAIN_ENTRY], cwd: ROOT });
}
