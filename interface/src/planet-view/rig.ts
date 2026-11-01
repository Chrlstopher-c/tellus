// Transitions de la vue : globe ↔ carte (morphing + caméra), fondu entre calques.
import * as THREE from 'three';

export type ViewMode = 'globe' | 'map';
export const GLOBE_CAM = new THREE.Vector3(0, 0.35, 6.0);
const MORPH_SECONDS = 1.6;
const LAYER_SECONDS = 0.45;

export const ease = (t: number): number => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2);

/** Distance caméra pour que la planisphère tienne entre les panneaux (marges en pixels). */
export function mapCameraZ(camera: THREE.PerspectiveCamera, width: number, height: number): number {
  const t = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
  return Math.max(0.8 / (t * (1 - 340 / height)), 1.6 / (t * camera.aspect * (1 - 592 / width)));
}

export class ViewRig {
  mode: ViewMode = 'globe';
  morph = 0;
  private from = 0;
  private to = 0;
  private t = 1;
  private camFrom = GLOBE_CAM.clone();
  private camTo = GLOBE_CAM.clone();
  private rotFrom = new THREE.Quaternion();
  layer = 0;
  layerPrev = 0;
  layerMix = 1;

  constructor(private readonly camera: THREE.PerspectiveCamera, private readonly group: THREE.Group) {}

  setMode(mode: ViewMode, width: number, height: number): void {
    if (mode === this.mode) return;
    this.mode = mode;
    this.from = this.morph;
    this.to = mode === 'map' ? 1 : 0;
    this.t = 0;
    this.camFrom = this.camera.position.clone();
    this.camTo = mode === 'map' ? new THREE.Vector3(0, 0, mapCameraZ(this.camera, width, height)) : GLOBE_CAM.clone();
    this.rotFrom = this.group.quaternion.clone();
  }

  setLayer(i: number): void {
    if (i === this.layer) return;
    this.layerPrev = this.layerMix < 1 ? this.layerPrev : this.layer;
    this.layer = i;
    this.layerMix = 0;
  }

  /** Avance les transitions ; renvoie vrai si un morphing est en cours (la caméra est alors pilotée ici). */
  update(dt: number, spin: number): boolean {
    this.layerMix = Math.min(1, this.layerMix + dt / LAYER_SECONDS);
    if (this.t >= 1) {
      if (this.mode === 'globe') this.group.rotateY(dt * spin);
      return false;
    }
    this.t = Math.min(1, this.t + dt / MORPH_SECONDS);
    const k = ease(this.t);
    this.morph = this.from + (this.to - this.from) * k;
    this.camera.position.lerpVectors(this.camFrom, this.camTo, k);
    if (this.mode === 'map') {
      this.group.quaternion.slerpQuaternions(this.rotFrom, new THREE.Quaternion(), Math.min(1, k * 1.4));
    }
    return true;
  }
}
