import { expect, test } from '@playwright/test';
import { launchTellus } from './launch';

test('sim-process : la fenêtre reçoit 40 962 cellules L6 et un instantané de 40 962 floats', async () => {
  const app = await launchTellus();
  try {
    const win = await app.firstWindow();
    const debug = win.getByTestId('sim-debug');
    await expect(debug).toHaveText('sim : 40962 cellules, tick 1');
    await expect(debug).toHaveAttribute('data-snapshot-length', '40962');
  } finally {
    await app.close();
  }
});
