// Vue planétaire : compose rendu, champs, objets, transitions et sélection. N'invente aucune donnée du monde.
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { FieldStore, type CellLookup, type CellSnapshot, type FieldSample } from './fields';
import {
  makeAtmosphere, makeClouds, makeMarker, makeStars, makeSurface, type Opacity, type Part, type SharedUniforms,
  type SurfaceUniforms,
} from './meshes';
import { GLOBE_CAM, ViewRig, type ViewMode } from './rig';

const SUN_VIEW = new THREE.Vector3(-0.62, 0.38, 0.68).normalize();
const MAP_W = 3.2;
const MAP_H = 1.6;

export interface PickResult extends FieldSample {
  lat: number;
  lon: number;
}

export interface FrameInput {
  dt: number;
  visualTime: number;
  spin: number;
  alpha: number;
}

export interface PlanetView {
  push(snapshot: CellSnapshot, seed: number): void;
  setMode(mode: ViewMode): void;
  setLayer(index: number): void;
  setClouds(on: boolean): void;
  pick(clientX: number, clientY: number): PickResult | null;
  frame(input: FrameInput): void;
  resize(): void;
  landFraction(): number;
  dispose(): void;
}

interface Parts {
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  controls: OrbitControls;
  group: THREE.Group;
  fields: FieldStore;
  shared: SharedUniforms;
  surface: Part<SurfaceUniforms>;
  clouds: Part<Opacity>;
  atmosphere: Part<Opacity>;
  marker: THREE.Mesh;
  pickSphere: THREE.Mesh;
  pickPlane: THREE.Mesh;
  rig: ViewRig;
}

function makeRenderer(canvas: HTMLCanvasElement): THREE.WebGLRenderer {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  return renderer;
}

function makeControls(camera: THREE.PerspectiveCamera, canvas: HTMLCanvasElement): OrbitControls {
  const controls = new OrbitControls(camera, canvas);
  Object.assign(controls, {
    enableDamping: true, dampingFactor: 0.06, minDistance: 1.6, maxDistance: 11, enablePan: false, rotateSpeed: 0.45,
  });
  return controls;
}

function invisible(geometry: THREE.BufferGeometry): THREE.Mesh {
  return new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({ visible: false }));
}

function assemble(canvas: HTMLCanvasElement, lookup: CellLookup, cellCount: number): Parts {
  const renderer = makeRenderer(canvas);
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(28, 1, 0.05, 400);
  camera.position.copy(GLOBE_CAM);
  const group = new THREE.Group();
  const fields = new FieldStore(renderer, lookup, cellCount);
  const shared: SharedUniforms = {
    uFields: { value: fields.target.texture }, uSun: { value: new THREE.Vector3() },
    uTime: { value: 0 }, uMorph: { value: 0 },
  };
  const parts: Parts = {
    renderer, scene, camera, group, fields, shared, controls: makeControls(camera, canvas),
    surface: makeSurface(shared, lookup.width, lookup.height), clouds: makeClouds(shared),
    atmosphere: makeAtmosphere(shared.uSun), marker: makeMarker(),
    pickSphere: invisible(new THREE.SphereGeometry(1, 64, 32)),
    pickPlane: invisible(new THREE.PlaneGeometry(MAP_W, MAP_H)),
    rig: new ViewRig(camera, group),
  };
  group.add(parts.surface.mesh, parts.clouds.mesh, parts.pickSphere, parts.marker);
  scene.add(group, parts.atmosphere.mesh, makeStars(), parts.pickPlane);
  return parts;
}

function uvFromHit(p: Parts, point: THREE.Vector3): THREE.Vector2 {
  if (p.rig.mode === 'map') return new THREE.Vector2(point.x / MAP_W + 0.5, point.y / MAP_H + 0.5);
  const d = p.group.worldToLocal(point.clone()).normalize();
  return new THREE.Vector2(Math.atan2(d.x, d.z) / (2 * Math.PI) + 0.5, Math.asin(d.y) / Math.PI + 0.5);
}

function pickAt(p: Parts, clientX: number, clientY: number): PickResult | null {
  const r = p.renderer.domElement.getBoundingClientRect();
  const ndc = new THREE.Vector2(((clientX - r.left) / r.width) * 2 - 1, -((clientY - r.top) / r.height) * 2 + 1);
  const ray = new THREE.Raycaster();
  ray.setFromCamera(ndc, p.camera);
  const hit = ray.intersectObject(p.rig.mode === 'globe' ? p.pickSphere : p.pickPlane)[0];
  if (!hit) return null;
  const uv = uvFromHit(p, hit.point);
  const sample = p.fields.sample(uv.x, uv.y);
  if (!sample) return null;
  p.marker.userData.uv = uv;
  p.marker.visible = true;
  return { ...sample, lat: (uv.y - 0.5) * 180, lon: (uv.x - 0.5) * 360 };
}

function placeMarker(p: Parts, time: number): void {
  const uv: unknown = p.marker.userData.uv;
  if (!p.marker.visible || !(uv instanceof THREE.Vector2)) return;
  const lon = (uv.x - 0.5) * 2 * Math.PI, lat = (uv.y - 0.5) * Math.PI;
  const d = new THREE.Vector3(Math.cos(lat) * Math.sin(lon), Math.sin(lat), Math.cos(lat) * Math.cos(lon));
  const flat = new THREE.Vector3((lon / Math.PI) * 1.6, (lat / Math.PI) * 1.6, 0.006);
  p.marker.position.copy(d.clone().multiplyScalar(1.016).lerp(flat, p.rig.morph));
  const n = d.clone().lerp(new THREE.Vector3(0, 0, 1), p.rig.morph).normalize();
  p.marker.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), n);
  p.marker.scale.setScalar(0.028 * (1 + 0.12 * Math.sin(time * 3)));
}

function syncUniforms(p: Parts, input: FrameInput): void {
  const u = p.surface.uniforms;
  u.uLayerA.value = p.rig.layerPrev;
  u.uLayerB.value = p.rig.layer;
  u.uLayerMix.value = p.rig.layerMix;
  u.uRot.value.setFromMatrix4(p.group.matrixWorld);
  p.shared.uTime.value = input.visualTime;
  p.shared.uMorph.value = p.rig.morph;
  p.shared.uSun.value.copy(SUN_VIEW).applyQuaternion(p.camera.quaternion);
  const atmo = 1 - Math.min(1, p.rig.morph * 2.5);
  p.atmosphere.uniforms.uOpacity.value = atmo;
  p.atmosphere.mesh.visible = atmo > 0.001;
  p.clouds.mesh.visible = p.clouds.uniforms.uOpacity.value > 0;
}

function renderFrame(p: Parts, input: FrameInput): void {
  if (p.rig.update(input.dt, input.spin)) p.controls.target.set(0, 0, 0);
  p.fields.resolve(input.alpha);
  syncUniforms(p, input);
  placeMarker(p, performance.now() / 1000);
  p.controls.update();
  p.renderer.render(p.scene, p.camera);
}

export function createPlanetView(canvas: HTMLCanvasElement, lookup: CellLookup, cellCount: number): PlanetView {
  const p = assemble(canvas, lookup, cellCount);
  return {
    push: (snapshot, seed) => { p.fields.setSeed(seed); p.fields.push(snapshot); },
    setMode: (mode) => {
      p.rig.setMode(mode, canvas.clientWidth, canvas.clientHeight);
      p.controls.enableRotate = mode === 'globe';
      p.controls.enablePan = mode === 'map';
    },
    setLayer: (i) => p.rig.setLayer(i),
    setClouds: (on) => { p.clouds.uniforms.uOpacity.value = on ? 1 : 0; },
    pick: (x, y) => pickAt(p, x, y),
    frame: (input) => renderFrame(p, input),
    resize: () => {
      p.renderer.setSize(canvas.clientWidth, canvas.clientHeight, false);
      p.camera.aspect = canvas.clientWidth / Math.max(1, canvas.clientHeight);
      p.camera.updateProjectionMatrix();
    },
    landFraction: () => p.fields.landFraction(),
    dispose: () => { p.controls.dispose(); p.renderer.dispose(); },
  };
}
