// Lissage des champs résolus : le relief simulé est linéaire par triangle (facettes) ; un flou gaussien séparable
// à l'échelle de la maille le rend continu, puis la passe finale ajoute le détail visuel sous la maille.
import * as THREE from 'three';
import { RESOLVE_VERT, SMOOTH_FRAG } from './shaders/resolve';

function workTarget(w: number, h: number): THREE.WebGLRenderTarget {
  return new THREE.WebGLRenderTarget(w, h, {
    type: THREE.HalfFloatType, format: THREE.RGBAFormat, depthBuffer: false, wrapS: THREE.RepeatWrapping,
    minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, generateMipmaps: false,
  });
}

interface SmoothUniforms {
  [name: string]: THREE.IUniform;
  uSrc: THREE.IUniform<THREE.Texture | null>;
  uStep: THREE.IUniform<THREE.Vector2>;
  uFinal: THREE.IUniform<number>;
  uSeed: THREE.IUniform<number>;
}

export class FieldSmoother {
  /** Cible de la passe de résolution brute. */
  readonly raw: THREE.WebGLRenderTarget;
  private readonly tmp: THREE.WebGLRenderTarget;
  private readonly uniforms: SmoothUniforms;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);

  constructor(private readonly w: number, private readonly h: number) {
    this.raw = workTarget(w, h);
    this.tmp = workTarget(w, h);
    this.uniforms = {
      uSrc: { value: null }, uStep: { value: new THREE.Vector2() }, uFinal: { value: 0 }, uSeed: { value: 0 },
    };
    const material = new THREE.ShaderMaterial({
      vertexShader: RESOLVE_VERT, fragmentShader: SMOOTH_FRAG, uniforms: this.uniforms,
    });
    this.scene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), material));
  }

  setSeed(seed: number): void {
    this.uniforms.uSeed.value = seed;
  }

  /** raw → (flou horizontal) → tmp → (flou vertical + détail) → target. */
  run(renderer: THREE.WebGLRenderer, target: THREE.WebGLRenderTarget): void {
    this.pass(renderer, this.raw, this.tmp, 1 / this.w, 0, 0);
    this.pass(renderer, this.tmp, target, 0, 1 / this.h, 1);
  }

  private pass(r: THREE.WebGLRenderer, src: THREE.WebGLRenderTarget, dst: THREE.WebGLRenderTarget,
    sx: number, sy: number, final: number): void {
    this.uniforms.uSrc.value = src.texture;
    this.uniforms.uStep.value.set(sx, sy);
    this.uniforms.uFinal.value = final;
    r.setRenderTarget(dst);
    r.render(this.scene, this.camera);
  }
}
