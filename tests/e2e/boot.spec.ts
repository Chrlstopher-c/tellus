import { expect, test } from '@playwright/test';
import { launchTellus } from './launch';

test('boot : la fenêtre s’ouvre, titre Tellus, #root rendu', async () => {
  const app = await launchTellus();
  try {
    const win = await app.firstWindow();
    await expect(win).toHaveTitle('Tellus');
    await expect(win.getByTestId('start')).toContainText('Tellus');
  } finally {
    await app.close();
  }
});
