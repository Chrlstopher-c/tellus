import { expect, test } from '@playwright/test';
import { launchTellus } from './launch';

test('renderer : data-renderer est webgpu ou webgl2, webgpu sur cette machine', async () => {
  const app = await launchTellus();
  try {
    const win = await app.firstWindow();
    const body = win.locator('body');
    await expect(body).toHaveAttribute('data-renderer', /^(webgpu|webgl2)$/);
    expect(await body.getAttribute('data-renderer')).toBe('webgpu');
  } finally {
    await app.close();
  }
});
