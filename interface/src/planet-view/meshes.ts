// Objets de la scène : surface et nuages morphables, atmosphère, étoiles, marqueur de sélection.
import * as THREE from 'three';
import { CLOUD_FRAG, CLOUD_VERT, ATMO_FRAG, ATMO_VERT } from './shaders/clouds';
import { SURFACE_FRAG, SURFACE_VERT } from './shaders/surface';

export interface SharedUniforms {
  [name: string]: THREE.IUniform;
  uFields: THREE.IUniform<THREE.Texture>;
  uSun: THREE.IUniform<THREE.Vector3>;
  uTime: THREE.IUniform<number>;
  uMorph: THREE.IUniform<number>;
}

export interface SurfaceUniforms extends SharedUniforms {
  uTexel: THREE.IUniform<THREE.Vector2>;
  uRot: THREE.IUniform<THREE.Matrix3>;
  uLayerA: THREE.IUniform<number>;
  uLayerB: THREE.IUniform<number>;
  uLayerMix: THREE.IUniform<number>;
}

export interface Opacity {
  [name: string]: THREE.IUniform;
  uOpacity: THREE.IUniform<number>;
}

export interface Part<U> {
  mesh: THREE.Mesh;
  uniforms: U;
}

export function makeSurface(shared: SharedUniforms, w: number, h: number): Part<SurfaceUniforms> {
  const uniforms: SurfaceUniforms = {
    ...shared, uTexel: { value: new THREE.Vector2(1 / w, 1 / h) }, uRot: { value: new THREE.Matrix3() },
    uLayerA: { value: 0 }, uLayerB: { value: 0 }, uLayerMix: { value: 0 },
  };
  const material = new THREE.ShaderMaterial({ vertexShader: SURFACE_VERT, fragmentShader: SURFACE_FRAG, uniforms });
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1, 512, 256), material);
  mesh.frustumCulled = false;
  return { mesh, uniforms };
}

export function makeClouds(shared: SharedUniforms): Part<Opacity> {
  const uniforms: Opacity = { ...shared, uOpacity: { value: 1 } };
  const material = new THREE.ShaderMaterial({
    vertexShader: CLOUD_VERT, fragmentShader: CLOUD_FRAG, transparent: true, depthWrite: false, uniforms,
  });
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1, 256, 128), material);
  mesh.frustumCulled = false;
  mesh.renderOrder = 1;
  return { mesh, uniforms };
}

export function makeAtmosphere(sun: THREE.IUniform<THREE.Vector3>): Part<Opacity> {
  const uniforms: Opacity = { uSun: sun, uOpacity: { value: 1 }, uLimb: { value: Math.sqrt(1 - 1 / 1.065 ** 2) } };
  const material = new THREE.ShaderMaterial({
    vertexShader: ATMO_VERT, fragmentShader: ATMO_FRAG, side: THREE.BackSide, transparent: true,
    blending: THREE.AdditiveBlending, depthWrite: false, uniforms,
  });
  return { mesh: new THREE.Mesh(new THREE.SphereGeometry(1.065, 128, 64), material), uniforms };
}

function starSprite(): THREE.CanvasTexture {
  const c = document.createElement('canvas');
  c.width = c.height = 32;
  const g = c.getContext('2d');
  if (g) {
    const grad = g.createRadialGradient(16, 16, 0, 16, 16, 16);
    grad.addColorStop(0, 'rgba(255,255,255,1)');
    grad.addColorStop(0.35, 'rgba(255,255,255,0.35)');
    grad.addColorStop(1, 'rgba(255,255,255,0)');
    g.fillStyle = grad;
    g.fillRect(0, 0, 32, 32);
  }
  return new THREE.CanvasTexture(c);
}

/** Étoiles de fond : purement décoratives, aléa fixe (n'influence jamais la simulation). */
export function makeStars(count = 2200): THREE.Points {
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  let s = 0x9e3779b9;
  const rnd = (): number => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 4294967296);
  for (let i = 0; i < count; i++) {
    const u = rnd() * 2 - 1, a = rnd() * Math.PI * 2, r = Math.sqrt(1 - u * u);
    pos.set([r * Math.cos(a) * 120, u * 120, r * Math.sin(a) * 120], i * 3);
    const b = Math.pow(rnd(), 10) * 1.6 + 0.0025, warm = rnd();
    col.set([b * (0.85 + 0.15 * warm), b * 0.9, b * (1.05 - 0.2 * warm)], i * 3);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  g.setAttribute('color', new THREE.BufferAttribute(col, 3));
  return new THREE.Points(g, new THREE.PointsMaterial({
    size: 2.2, sizeAttenuation: false, vertexColors: true, depthWrite: false, map: starSprite(),
    transparent: true, blending: THREE.AdditiveBlending,
  }));
}

export function makeMarker(): THREE.Mesh {
  const m = new THREE.Mesh(
    new THREE.RingGeometry(0.62, 1, 48),
    new THREE.MeshBasicMaterial({
      color: 0xf4ede2, transparent: true, opacity: 0.95, depthWrite: false, side: THREE.DoubleSide,
    }),
  );
  m.visible = false;
  m.renderOrder = 2;
  return m;
}
