// Champs de la planète côté GPU : table texel → cellules, deux instantanés par cellule, texture résolue.
import * as THREE from 'three';
import type { CellLookup } from '../sim-bridge/protocol';
import { RESOLVE_FRAG, RESOLVE_VERT } from './shaders/resolve';

export const CELLS_WIDTH = 1024;
export const FIELD_CHANNELS = 4;

export type { CellLookup } from '../sim-bridge/protocol';

/** Instantané : par cellule altitude (m), température (°C), précipitations (mm/an), couvert végétal (0-1). */
export interface CellSnapshot {
  cellCount: number;
  fields: Float32Array;
  simYears: number;
}

export interface FieldSample {
  cell: number;
  elevation: number;
  temperature: number;
  precipitation: number;
}

function floatTexture(data: Float32Array, w: number, h: number, format: THREE.PixelFormat): THREE.DataTexture {
  const t = new THREE.DataTexture(data, w, h, format, THREE.FloatType);
  t.minFilter = THREE.NearestFilter;
  t.magFilter = THREE.NearestFilter;
  t.needsUpdate = true;
  return t;
}

interface CellBuffer {
  data: Float32Array;
  texture: THREE.DataTexture;
}

function cellBuffer(cellCount: number): CellBuffer {
  const rows = Math.ceil(cellCount / CELLS_WIDTH);
  const data = new Float32Array(CELLS_WIDTH * rows * FIELD_CHANNELS);
  return { data, texture: floatTexture(data, CELLS_WIDTH, rows, THREE.RGBAFormat) };
}

function resolveTarget(w: number, h: number, maxAniso: number): THREE.WebGLRenderTarget {
  const rt = new THREE.WebGLRenderTarget(w, h, {
    type: THREE.HalfFloatType, format: THREE.RGBAFormat, depthBuffer: false, wrapS: THREE.RepeatWrapping,
    minFilter: THREE.LinearMipmapLinearFilter, magFilter: THREE.LinearFilter, generateMipmaps: true,
  });
  rt.texture.anisotropy = maxAniso;
  return rt;
}

interface ResolveUniforms {
  [name: string]: THREE.IUniform;
  uLookupIds: THREE.IUniform<THREE.Texture>;
  uLookupWeights: THREE.IUniform<THREE.Texture>;
  uCellsPrev: THREE.IUniform<THREE.Texture>;
  uCellsNext: THREE.IUniform<THREE.Texture>;
  uAlpha: THREE.IUniform<number>;
  uCellsWidth: THREE.IUniform<number>;
  uSeed: THREE.IUniform<number>;
}

export class FieldStore {
  readonly target: THREE.WebGLRenderTarget;
  private readonly uniforms: ResolveUniforms;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private prev: CellBuffer;
  private next: CellBuffer;
  private latest: CellSnapshot | null = null;

  constructor(private readonly renderer: THREE.WebGLRenderer, private readonly lookup: CellLookup, cellCount: number) {
    this.target = resolveTarget(lookup.width, lookup.height, renderer.capabilities.getMaxAnisotropy());
    this.prev = cellBuffer(cellCount);
    this.next = cellBuffer(cellCount);
    this.uniforms = {
      uLookupIds: { value: floatTexture(lookup.ids, lookup.width, lookup.height, THREE.RGBAFormat) },
      uLookupWeights: { value: floatTexture(lookup.weights, lookup.width, lookup.height, THREE.RGFormat) },
      uCellsPrev: { value: this.prev.texture }, uCellsNext: { value: this.next.texture },
      uAlpha: { value: 1 }, uCellsWidth: { value: CELLS_WIDTH }, uSeed: { value: 0 },
    };
    const material = new THREE.ShaderMaterial({
      vertexShader: RESOLVE_VERT, fragmentShader: RESOLVE_FRAG, uniforms: this.uniforms,
    });
    this.scene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), material));
  }

  /** Nouvel instantané : l'ancien « suivant » devient « précédent », l'interpolation repart de 0. */
  push(snapshot: CellSnapshot): void {
    [this.prev, this.next] = [this.next, this.prev];
    this.next.data.set(snapshot.fields);
    this.next.texture.needsUpdate = true;
    if (!this.latest) {
      this.prev.data.set(snapshot.fields);
      this.prev.texture.needsUpdate = true;
    }
    this.uniforms.uCellsPrev.value = this.prev.texture;
    this.uniforms.uCellsNext.value = this.next.texture;
    this.latest = snapshot;
  }

  setSeed(seed: number): void {
    this.uniforms.uSeed.value = (seed % 1000) / 7.3;
  }

  resolve(alpha: number): void {
    this.uniforms.uAlpha.value = Math.min(1, Math.max(0, alpha));
    this.renderer.setRenderTarget(this.target);
    this.renderer.render(this.scene, this.camera);
    this.renderer.setRenderTarget(null);
  }

  /** Valeurs de la cellule dominante sous une coordonnée équirectangulaire (0..1), lues côté CPU. */
  sample(u: number, v: number): FieldSample | null {
    if (!this.latest) return null;
    const x = Math.min(this.lookup.width - 1, Math.floor(u * this.lookup.width));
    const y = Math.min(this.lookup.height - 1, Math.floor(v * this.lookup.height));
    const t = y * this.lookup.width + x;
    const w0 = this.lookup.weights[t * 2] ?? 0, w1 = this.lookup.weights[t * 2 + 1] ?? 0;
    const slot = w0 >= w1 && w0 >= 1 - w0 - w1 ? 0 : w1 >= 1 - w0 - w1 ? 1 : 2;
    const cell = Math.round(this.lookup.ids[t * 4 + slot] ?? 0);
    const f = this.latest.fields;
    const at = (k: number): number => f[cell * 4 + k] ?? 0;
    return { cell, elevation: at(0), temperature: at(1), precipitation: at(2) };
  }

  landFraction(): number {
    if (!this.latest) return 0;
    const f = this.latest.fields;
    let land = 0;
    for (let i = 0; i < this.latest.cellCount; i++) if ((f[i * 4] ?? 0) > 0) land++;
    return land / this.latest.cellCount;
  }
}
