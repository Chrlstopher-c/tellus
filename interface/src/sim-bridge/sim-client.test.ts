import { expect, it } from 'vitest';
import type { SimCommand, SimReply } from './protocol';
import { createSimClient, type SimPort } from './sim-client';

class FakePort implements SimPort {
  readonly sent: SimCommand[] = [];
  onmessage: ((event: MessageEvent<SimReply>) => void) | null = null;
  closed = false;

  postMessage(message: SimCommand): void {
    this.sent.push(message);
  }

  close(): void {
    this.closed = true;
  }

  reply(message: SimReply): void {
    this.onmessage?.(new MessageEvent('message', { data: message }));
  }
}

it('envoie create et résout avec le nombre de cellules', async () => {
  const port = new FakePort();
  const pending = createSimClient(port).createWorld(42, 6);
  expect(port.sent).toEqual([{ type: 'create', id: 1, seed: 42, level: 6 }]);
  port.reply({ id: 1, ok: true, result: { type: 'create', cells: 40962 } });
  await expect(pending).resolves.toBe(40962);
});

it('envoie step et résout avec le temps simulé total', async () => {
  const port = new FakePort();
  const pending = createSimClient(port).step(2.5);
  expect(port.sent).toEqual([{ type: 'step', id: 1, dtYears: 2.5 }]);
  port.reply({ id: 1, ok: true, result: { type: 'step', time: 2.5 } });
  await expect(pending).resolves.toBe(2.5);
});

it('résout snapshotElevation avec les valeurs reçues', async () => {
  const port = new FakePort();
  const pending = createSimClient(port).snapshotElevation();
  port.reply({ id: 1, ok: true, result: { type: 'snapshot', values: new Float32Array([1.5, 2.5]) } });
  const snapshot = await pending;
  expect(snapshot).toBeInstanceOf(Float32Array);
  expect(Array.from(snapshot)).toEqual([1.5, 2.5]);
});

it('rejette avec le message d’erreur de la simulation', async () => {
  const port = new FakePort();
  const pending = createSimClient(port).step(-1);
  port.reply({ id: 1, ok: false, error: 'durée invalide' });
  await expect(pending).rejects.toThrow('durée invalide');
});

it('rejette si la réponse n’est pas du type attendu', async () => {
  const port = new FakePort();
  const pending = createSimClient(port).step(1);
  port.reply({ id: 1, ok: true, result: { type: 'create', cells: 12 } });
  await expect(pending).rejects.toThrow('« create » reçue');
});

it('associe chaque réponse à sa requête même si elles arrivent dans le désordre', async () => {
  const port = new FakePort();
  const client = createSimClient(port);
  const first = client.step(1);
  const second = client.step(2);
  port.reply({ id: 2, ok: true, result: { type: 'step', time: 3 } });
  port.reply({ id: 1, ok: true, result: { type: 'step', time: 1 } });
  await expect(first).resolves.toBe(1);
  await expect(second).resolves.toBe(3);
});

it('ignore une réponse sans requête en attente', () => {
  const port = new FakePort();
  createSimClient(port);
  expect(() => port.reply({ id: 99, ok: true, result: { type: 'step', time: 0 } })).not.toThrow();
});

it('dispose ferme le port et rejette les requêtes en attente', async () => {
  const port = new FakePort();
  const client = createSimClient(port);
  const pending = client.step(1);
  client.dispose();
  expect(port.closed).toBe(true);
  await expect(pending).rejects.toThrow('client de simulation fermé');
});
