import { describe, expect, it } from 'vitest';
import type { SimCommand, SimMessage } from './protocol';
import { createSimClient, type SimPort } from './sim-client';

class FakePort implements SimPort {
  onmessage: ((event: MessageEvent<SimMessage>) => void) | null = null;
  sent: SimCommand[] = [];
  closed = false;
  postMessage(message: SimCommand): void {
    this.sent.push(message);
  }
  close(): void {
    this.closed = true;
  }
  emit(message: SimMessage): void {
    this.onmessage?.(new MessageEvent('message', { data: message }));
  }
}

const LOOKUP = { width: 2, height: 1, ids: new Float32Array(8), weights: new Float32Array(4) };

describe('sim-client', () => {
  it('createWorld envoie la commande et renvoie le monde', async () => {
    const port = new FakePort();
    const client = createSimClient(port);
    const pending = client.createWorld({ config: { source: { kind: 'body', name: 'Terre' } }, level: 6 });
    const sent = port.sent[0];
    expect(sent?.type).toBe('create');
    const world = { cells: 40962, seed: 1, name: 'Terre', lookup: LOOKUP };
    port.emit({ kind: 'reply', id: sent?.id ?? 0, ok: true, result: { type: 'create', world } });
    await expect(pending).resolves.toEqual(world);
  });

  it('propage une erreur de la simulation', async () => {
    const port = new FakePort();
    const pending = createSimClient(port).inspect(3);
    port.emit({ kind: 'reply', id: port.sent[0]?.id ?? 0, ok: false, error: 'cellule hors grille' });
    await expect(pending).rejects.toThrow('cellule hors grille');
  });

  it('distribue les trames aux abonnés et permet de se désabonner', () => {
    const port = new FakePort();
    const client = createSimClient(port);
    const seen: number[] = [];
    const off = client.onFrame((f) => seen.push(f.simYears));
    const vitals = { o2Percent: 0, co2Ppm: 0, meanTempC: 0, species: 0, biomassGtC: 0, era: '' };
    port.emit({ kind: 'frame', simYears: 5, effectiveRatio: 1, fields: new Float32Array(4), vitals });
    off();
    port.emit({ kind: 'frame', simYears: 6, effectiveRatio: 1, fields: new Float32Array(4), vitals });
    expect(seen).toEqual([5]);
  });

  it('dispose ferme le port et rejette les requêtes en attente', async () => {
    const port = new FakePort();
    const client = createSimClient(port);
    const pending = client.control(true, 'year');
    client.dispose();
    expect(port.closed).toBe(true);
    await expect(pending).rejects.toThrow('fermé');
  });
});
