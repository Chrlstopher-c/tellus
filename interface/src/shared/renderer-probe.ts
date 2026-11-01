import { rendererLogger } from './logger';

export type RendererKind = 'webgpu' | 'webgl2' | 'none';

export interface AdapterInfo {
  readonly vendor: string;
  readonly architecture: string;
  readonly device: string;
  readonly description: string;
}

export interface RendererProbe {
  readonly kind: RendererKind;
  readonly adapter: AdapterInfo | null;
}

export interface GpuAdapterLike {
  readonly info?: Partial<AdapterInfo>;
}

export interface GpuLike {
  requestAdapter(): Promise<GpuAdapterLike | null>;
}

export interface NavigatorLike {
  readonly gpu?: GpuLike;
}

export async function probeRenderer(
  nav: NavigatorLike = globalThis.navigator,
  canWebGL2: () => boolean = hasWebGL2,
): Promise<RendererProbe> {
  const adapter = await probeWebGPU(nav);
  if (adapter) return { kind: 'webgpu', adapter };
  if (canWebGL2()) return { kind: 'webgl2', adapter: null };
  return { kind: 'none', adapter: null };
}

async function probeWebGPU(nav: NavigatorLike): Promise<AdapterInfo | null> {
  if (!nav.gpu) return null;
  try {
    const adapter = await nav.gpu.requestAdapter();
    if (!adapter) {
      rendererLogger.warn('WebGPU présent mais aucun adaptateur');
      return null;
    }
    return toAdapterInfo(adapter.info);
  } catch (error: unknown) {
    rendererLogger.warn({ err: error }, 'requestAdapter a échoué');
    return null;
  }
}

function toAdapterInfo(info: Partial<AdapterInfo> | undefined): AdapterInfo {
  return {
    vendor: info?.vendor ?? '',
    architecture: info?.architecture ?? '',
    device: info?.device ?? '',
    description: info?.description ?? '',
  };
}

function hasWebGL2(): boolean {
  try {
    return document.createElement('canvas').getContext('webgl2') !== null;
  } catch (error: unknown) {
    rendererLogger.warn({ err: error }, 'WebGL2 indisponible');
    return false;
  }
}
