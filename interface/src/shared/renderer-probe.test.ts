import { describe, expect, it } from 'vitest';
import { probeRenderer, type AdapterInfo, type GpuLike } from './renderer-probe';

const ADAPTER: AdapterInfo = {
  vendor: 'nvidia',
  architecture: 'ampere',
  device: 'rtx 3060',
  description: 'NVIDIA GeForce RTX 3060',
};

function gpuReturning(info: Partial<AdapterInfo> | null): GpuLike {
  return { requestAdapter: () => Promise.resolve(info === null ? null : { info }) };
}

describe('probeRenderer', () => {
  it('renvoie webgpu avec les infos de l’adaptateur', async () => {
    const probe = await probeRenderer({ gpu: gpuReturning(ADAPTER) }, () => true);
    expect(probe).toEqual({ kind: 'webgpu', adapter: ADAPTER });
  });

  it('bascule sur webgl2 quand WebGPU n’a pas d’adaptateur', async () => {
    const probe = await probeRenderer({ gpu: gpuReturning(null) }, () => true);
    expect(probe).toEqual({ kind: 'webgl2', adapter: null });
  });

  it('renvoie none sans WebGPU ni WebGL2', async () => {
    const probe = await probeRenderer({}, () => false);
    expect(probe).toEqual({ kind: 'none', adapter: null });
  });

  it('traite une erreur de requestAdapter comme une absence de WebGPU', async () => {
    const failing: GpuLike = { requestAdapter: () => Promise.reject(new Error('gpu perdu')) };
    const probe = await probeRenderer({ gpu: failing }, () => true);
    expect(probe).toEqual({ kind: 'webgl2', adapter: null });
  });
});
