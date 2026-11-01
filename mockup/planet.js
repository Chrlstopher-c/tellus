// Scène 3D : cuisson des champs, globe ↔ carte, nuages, atmosphère, étoiles, sonde de clic.
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { BAKE_VERT, BAKE_FRAG } from './shaders/bake.js';
import { SURFACE_VERT, SURFACE_FRAG } from './shaders/surface.js';
import { CLOUD_VERT, CLOUD_FRAG, ATMO_VERT, ATMO_FRAG } from './shaders/clouds.js';

const FIELD_W = 4096;
const FIELD_H = 2048;
const SUN_VIEW = new THREE.Vector3(-0.62, 0.38, 0.68).normalize();
const ease = (t) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2);

export function createPlanet(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(28, 1, 0.05, 400);
  camera.position.set(0, 0.35, 4.6);
  const controls = new OrbitControls(camera, canvas);
  Object.assign(controls, { enableDamping: true, dampingFactor: 0.06, minDistance: 1.6, maxDistance: 9, enablePan: false });
  controls.rotateSpeed = 0.45;

  const fieldsRT = new THREE.WebGLRenderTarget(FIELD_W, FIELD_H, {
    type: THREE.FloatType, format: THREE.RGBAFormat, minFilter: THREE.LinearFilter,
    magFilter: THREE.LinearFilter, wrapS: THREE.RepeatWrapping, depthBuffer: false,
  });
  const bakeMat = new THREE.ShaderMaterial({
    vertexShader: BAKE_VERT, fragmentShader: BAKE_FRAG,
    uniforms: { uSeed: { value: 48.213 }, uSea: { value: 0.11 }, uTempOffset: { value: 0 }, uWet: { value: 1 } },
  });
  const bakeScene = new THREE.Scene();
  bakeScene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), bakeMat));
  const bakeCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);

  const group = new THREE.Group();
  scene.add(group);
  const shared = {
    uFields: { value: fieldsRT.texture }, uSun: { value: new THREE.Vector3() }, uTime: { value: 0 },
  };
  const surfaceMat = new THREE.ShaderMaterial({
    vertexShader: SURFACE_VERT, fragmentShader: SURFACE_FRAG,
    uniforms: {
      ...shared, uTexel: { value: new THREE.Vector2(1 / FIELD_W, 1 / FIELD_H) }, uMorph: { value: 0 },
      uLayer: { value: 0 }, uCloudsOn: { value: 1 }, uRot: { value: new THREE.Matrix3() },
    },
  });
  const surface = new THREE.Mesh(new THREE.PlaneGeometry(1, 1, 512, 256), surfaceMat);
  surface.frustumCulled = false;
  group.add(surface);

  const cloudMat = new THREE.ShaderMaterial({
    vertexShader: CLOUD_VERT, fragmentShader: CLOUD_FRAG, transparent: true, depthWrite: false,
    uniforms: { ...shared, uOpacity: { value: 1 } },
  });
  const clouds = new THREE.Mesh(new THREE.SphereGeometry(1.012, 160, 96), cloudMat);
  group.add(clouds);

  const atmoMat = new THREE.ShaderMaterial({
    vertexShader: ATMO_VERT, fragmentShader: ATMO_FRAG, side: THREE.BackSide, transparent: true,
    blending: THREE.AdditiveBlending, depthWrite: false,
    uniforms: { uSun: shared.uSun, uOpacity: { value: 1 }, uLimb: { value: Math.sqrt(1 - 1 / 1.065 ** 2) } },
  });
  const atmosphere = new THREE.Mesh(new THREE.SphereGeometry(1.065, 128, 64), atmoMat);
  scene.add(atmosphere);
  scene.add(makeStars());

  const pickSphere = new THREE.Mesh(new THREE.SphereGeometry(1, 64, 32), new THREE.MeshBasicMaterial({ visible: false }));
  const pickPlane = new THREE.Mesh(new THREE.PlaneGeometry(3.2, 1.6), new THREE.MeshBasicMaterial({ visible: false }));
  group.add(pickSphere);
  scene.add(pickPlane);
  const marker = makeMarker();
  group.add(marker);

  const state = { morph: 0, morphFrom: 0, morphTo: 0, morphT: 1, spin: 0, mode: 'globe', savedQuat: null };

  function bake() {
    renderer.setRenderTarget(fieldsRT);
    renderer.render(bakeScene, bakeCam);
    renderer.setRenderTarget(null);
  }

  function resize() {
    const w = canvas.clientWidth, h = canvas.clientHeight;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  function setMode(mode) {
    if (mode === state.mode) return;
    state.mode = mode;
    state.morphFrom = state.morph;
    state.morphTo = mode === 'map' ? 1 : 0;
    state.morphT = 0;
    state.camFrom = camera.position.clone();
    state.camTo = mode === 'map' ? new THREE.Vector3(0, 0, 4.2) : new THREE.Vector3(0, 0.35, 4.6);
    state.rotFrom = group.quaternion.clone();
    controls.enableRotate = mode === 'globe';
    controls.enablePan = mode === 'map';
  }

  function sample(uv) {
    const buf = new Float32Array(4);
    const x = Math.min(FIELD_W - 1, Math.floor(uv.x * FIELD_W));
    const y = Math.min(FIELD_H - 1, Math.floor(uv.y * FIELD_H));
    renderer.readRenderTargetPixels(fieldsRT, x, y, 1, 1, buf);
    return { elevation: buf[0], temperature: buf[1], precipitation: buf[2] };
  }

  const ray = new THREE.Raycaster();
  function pick(clientX, clientY) {
    const r = canvas.getBoundingClientRect();
    const ndc = new THREE.Vector2(((clientX - r.left) / r.width) * 2 - 1, -((clientY - r.top) / r.height) * 2 + 1);
    ray.setFromCamera(ndc, camera);
    const target = state.mode === 'globe' ? pickSphere : pickPlane;
    const hit = ray.intersectObject(target)[0];
    if (!hit) return null;
    let uv, dir;
    if (state.mode === 'globe') {
      dir = group.worldToLocal(hit.point.clone()).normalize();
      uv = new THREE.Vector2(Math.atan2(dir.x, dir.z) / (2 * Math.PI) + 0.5, Math.asin(dir.y) / Math.PI + 0.5);
    } else {
      uv = new THREE.Vector2(hit.point.x / 3.2 + 0.5, hit.point.y / 1.6 + 0.5);
    }
    placeMarker(uv);
    const lat = (uv.y - 0.5) * 180, lon = (uv.x - 0.5) * 360;
    return { lat, lon, ...sample(uv) };
  }

  function placeMarker(uv) {
    marker.userData.uv = uv;
    marker.visible = true;
  }

  function updateMarker(time) {
    if (!marker.visible) return;
    const { uv } = marker.userData;
    const lon = (uv.x - 0.5) * 2 * Math.PI, lat = (uv.y - 0.5) * Math.PI;
    const d = new THREE.Vector3(Math.cos(lat) * Math.sin(lon), Math.sin(lat), Math.cos(lat) * Math.cos(lon));
    const flat = new THREE.Vector3((lon / Math.PI) * 1.6, (lat / Math.PI) * 1.6, 0.002);
    marker.position.copy(d.clone().multiplyScalar(1.004).lerp(flat, state.morph));
    const n = d.clone().lerp(new THREE.Vector3(0, 0, 1), state.morph).normalize();
    marker.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), n);
    marker.scale.setScalar(0.028 * (1 + 0.12 * Math.sin(time * 3)));
  }

  function frame(dt, simTime, spinRate) {
    if (state.morphT < 1) {
      state.morphT = Math.min(1, state.morphT + dt / 1.6);
      const k = ease(state.morphT);
      state.morph = state.morphFrom + (state.morphTo - state.morphFrom) * k;
      camera.position.lerpVectors(state.camFrom, state.camTo, k);
      if (state.mode === 'map') group.quaternion.slerpQuaternions(state.rotFrom, new THREE.Quaternion(), Math.min(1, k * 1.4));
      controls.target.set(0, 0, 0);
    } else if (state.mode === 'globe') {
      group.rotateY(dt * spinRate);
    }
    shared.uTime.value = simTime;
    surfaceMat.uniforms.uMorph.value = state.morph;
    surfaceMat.uniforms.uRot.value.setFromMatrix4(group.matrixWorld);
    const fade = 1 - Math.min(1, state.morph * 2.5);
    cloudMat.uniforms.uOpacity.value = fade * surfaceMat.uniforms.uCloudsOn.value;
    atmoMat.uniforms.uOpacity.value = fade;
    clouds.visible = atmosphere.visible = fade > 0.001;
    shared.uSun.value.copy(SUN_VIEW).applyQuaternion(camera.quaternion);
    updateMarker(performance.now() / 1000);
    controls.update();
    renderer.render(scene, camera);
  }

  return {
    renderer, bake, resize, setMode, pick, frame,
    setLayer: (i) => { surfaceMat.uniforms.uLayer.value = i; },
    setClouds: (on) => { surfaceMat.uniforms.uCloudsOn.value = on ? 1 : 0; },
    setWorld: (p) => { Object.entries(p).forEach(([k, v]) => { bakeMat.uniforms[k].value = v; }); bake(); },
    landFraction: () => estimateLand(renderer, fieldsRT),
  };
}

function makeStars() {
  const n = 5200, pos = new Float32Array(n * 3), col = new Float32Array(n * 3);
  for (let i = 0; i < n; i++) {
    const u = Math.random() * 2 - 1, a = Math.random() * Math.PI * 2, s = Math.sqrt(1 - u * u);
    pos.set([s * Math.cos(a) * 120, u * 120, s * Math.sin(a) * 120], i * 3);
    const b = Math.pow(Math.random(), 6) * 0.9 + 0.08, warm = Math.random();
    col.set([b * (0.85 + 0.15 * warm), b * 0.9, b * (1.05 - 0.2 * warm)], i * 3);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  g.setAttribute('color', new THREE.BufferAttribute(col, 3));
  return new THREE.Points(g, new THREE.PointsMaterial({ size: 1.4, sizeAttenuation: false, vertexColors: true, depthWrite: false }));
}

function makeMarker() {
  const m = new THREE.Mesh(
    new THREE.RingGeometry(0.62, 1, 48),
    new THREE.MeshBasicMaterial({ color: 0xf4ede2, transparent: true, opacity: 0.95, depthWrite: false, side: THREE.DoubleSide }),
  );
  m.visible = false;
  return m;
}

function estimateLand(renderer, rt) {
  const w = 256, h = 128, buf = new Float32Array(4);
  let land = 0, total = 0;
  for (let j = 0; j < h; j++) {
    const lat = ((j + 0.5) / h - 0.5) * Math.PI, wgt = Math.cos(lat);
    for (let i = 0; i < w; i += 4) {
      renderer.readRenderTargetPixels(rt, Math.floor((i / w) * rt.width), Math.floor(((j + 0.5) / h) * rt.height), 1, 1, buf);
      total += wgt;
      if (buf[0] > 0) land += wgt;
    }
  }
  return land / total;
}
