import { expect, test, type Page } from '@playwright/test';
import { launchTellus } from './launch';

async function fps(win: Page): Promise<number> {
  return win.evaluate(() => new Promise<number>((resolve) => {
    let n = 0;
    const t0 = performance.now();
    const tick = (): void => {
      n += 1;
      if (performance.now() - t0 < 2000) requestAnimationFrame(tick);
      else resolve(n / 2);
    };
    requestAnimationFrame(tick);
  }));
}

test('app : départ → monde créé → temps qui avance → calques → inspecteur → carte, sans erreur', async () => {
  const app = await launchTellus();
  const errors: string[] = [];
  try {
    const win = await app.firstWindow();
    win.on('pageerror', (e) => errors.push(e.message));
    win.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
    await win.getByTestId('launch').click();
    await expect(win.locator('main')).toHaveAttribute('data-phase', 'running', { timeout: 60_000 });
    await expect(win.getByTestId('planet')).toBeVisible();
    const clock = win.getByTestId('clock');
    const first = await clock.textContent();
    await expect.poll(async () => clock.textContent(), { timeout: 10_000 }).not.toBe(first);
    await expect(win.getByTestId('vitals')).toContainText('CO₂');
    await win.getByRole('button', { name: 'Température' }).click();
    const box = await win.getByTestId('planet').boundingBox();
    if (box) await win.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    await expect(win.getByTestId('inspector')).toContainText('Région sélectionnée', { timeout: 10_000 });
    await win.locator('[data-mode=map]').click();
    expect(await fps(win)).toBeGreaterThan(50);
    expect(errors).toEqual([]);
  } finally {
    await app.close();
  }
});
