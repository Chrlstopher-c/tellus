// Scène 3D : cuisson des champs, globe ↔ carte, nuages, atmosphère, étoiles, sonde de clic.
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { BAKE_VERT, BAKE_FRAG } from './shaders/bake.js';
import { SURFACE_VERT, SURFACE_FRAG } from './shaders/surface.js';
import { CLOUD_VERT, CLOUD_FRAG, ATMO_VERT, ATMO_FRAG } from './shaders/clouds.js';

const FIELD_W = 4096;
const FIELD_H = 2048;
const GLOBE_CAM = new THREE.Vector3(0, 0.35, 6.0);
const SUN_VIEW = new THREE.Vector3(-0.62, 0.38, 0.68).normalize();
const ease = (t) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2);

function fieldTarget(w, h, mip) {
  return new THREE.WebGLRenderTarget(w, h, {
    type: THREE.HalfFloatType, format: THREE.RGBAFormat, depthBuffer: false, wrapS: THREE.RepeatWrapping,
    minFilter: mip ? THREE.LinearMipmapLinearFilter : THREE.NearestFilter, magFilter: THREE.LinearFilter,
    generateMipmaps: mip,
  });
}

export function createPlanet(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(28, 1, 0.05, 400);
  camera.position.copy(GLOBE_CAM);
  const controls = new OrbitControls(camera, canvas);
  Object.assign(controls, { enableDamping: true, dampingFactor: 0.06, minDistance: 1.6, maxDistance: 11, enablePan: false, rotateSpeed: 0.45 });

  const fieldsRT = fieldTarget(FIELD_W, FIELD_H, true);
  fieldsRT.texture.anisotropy = renderer.capabilities.getMaxAnisotropy();
  const probeRT = fieldTarget(256, 128, false);
  const bakeMat = new THREE.ShaderMaterial({
    vertexShader: BAKE_VERT, fragmentShader: BAKE_FRAG,
    uniforms: { uSeed: { value: 48.213 }, uSea: { value: 0.11 }, uTempOffset: { value: 0 }, uWet: { value: 1 } },
  });
  const bakeScene = new THREE.Scene();
  bakeScene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), bakeMat));
  const bakeCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);

  const group = new THREE.Group();
  scene.add(group);
  const shared = { uFields: { value: fieldsRT.texture }, uSun: { value: new THREE.Vector3() }, uTime: { value: 0 }, uMorph: { value: 0 } };
  const surfaceMat = new THREE.ShaderMaterial({
    vertexShader: SURFACE_VERT, fragmentShader: SURFACE_FRAG,
    uniforms: {
      ...shared, uTexel: { value: new THREE.Vector2(1 / FIELD_W, 1 / FIELD_H) }, uRot: { value: new THREE.Matrix3() },
      uLayerA: { value: 0 }, uLayerB: { value: 0 }, uLayerMix: { value: 0 },
    },
  });
  const surface = new THREE.Mesh(new THREE.PlaneGeometry(1, 1, 512, 256), surfaceMat);
  surface.frustumCulled = false;
  group.add(surface);

  const cloudMat = new THREE.ShaderMaterial({
    vertexShader: CLOUD_VERT, fragmentShader: CLOUD_FRAG, transparent: true, depthWrite: false,
    uniforms: { ...shared, uOpacity: { value: 1 } },
  });
  const clouds = new THREE.Mesh(new THREE.PlaneGeometry(1, 1, 256, 128), cloudMat);
  clouds.frustumCulled = false;
  clouds.renderOrder = 1;
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

  const state = { morph: 0, morphFrom: 0, morphTo: 0, morphT: 1, mode: 'globe', layer: 0, layerT: 1 };

  function bake() {
    renderer.setRenderTarget(fieldsRT);
    renderer.render(bakeScene, bakeCam);
    renderer.setRenderTarget(probeRT);
    renderer.render(bakeScene, bakeCam);
    renderer.setRenderTarget(null);
  }

  function resize() {
    const w = canvas.clientWidth, h = canvas.clientHeight;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  function mapCamera() {
    const t = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
    const z = Math.max(0.8 / (t * (1 - 340 / innerHeight)), 1.6 / (t * camera.aspect * (1 - 592 / innerWidth)));
    return new THREE.Vector3(0, 0, z);
  }

  function setMode(mode) {
    if (mode === state.mode) return;
    Object.assign(state, {
      mode, morphFrom: state.morph, morphTo: mode === 'map' ? 1 : 0, morphT: 0,
      camFrom: camera.position.clone(), camTo: mode === 'map' ? mapCamera() : GLOBE_CAM.clone(), rotFrom: group.quaternion.clone(),
    });
    controls.enableRotate = mode === 'globe';
    controls.enablePan = mode === 'map';
  }

  function setLayer(i) {
    const u = surfaceMat.uniforms;
    u.uLayerA.value = state.layerT < 1 ? u.uLayerB.value : state.layer;
    u.uLayerB.value = i;
    u.uLayerMix.value = 0;
    state.layer = i;
    state.layerT = 0;
  }

  function sample(uv) {
    const buf = new Uint16Array(4);
    const x = Math.min(FIELD_W - 1, Math.floor(uv.x * FIELD_W));
    const y = Math.min(FIELD_H - 1, Math.floor(uv.y * FIELD_H));
    renderer.readRenderTargetPixels(fieldsRT, x, y, 1, 1, buf);
    const f = Array.from(buf, (h) => THREE.DataUtils.fromHalfFloat(h));
    return { elevation: f[0], temperature: f[1], precipitation: f[2] };
  }

  const ray = new THREE.Raycaster();
  function pick(clientX, clientY) {
    const r = canvas.getBoundingClientRect();
    const ndc = new THREE.Vector2(((clientX - r.left) / r.width) * 2 - 1, -((clientY - r.top) / r.height) * 2 + 1);
    ray.setFromCamera(ndc, camera);
    const hit = ray.intersectObject(state.mode === 'globe' ? pickSphere : pickPlane)[0];
    if (!hit) return null;
    let uv;
    if (state.mode === 'globe') {
      const dir = group.worldToLocal(hit.point.clone()).normalize();
      uv = new THREE.Vector2(Math.atan2(dir.x, dir.z) / (2 * Math.PI) + 0.5, Math.asin(dir.y) / Math.PI + 0.5);
    } else {
      uv = new THREE.Vector2(hit.point.x / 3.2 + 0.5, hit.point.y / 1.6 + 0.5);
    }
    marker.userData.uv = uv;
    marker.visible = true;
    return { lat: (uv.y - 0.5) * 180, lon: (uv.x - 0.5) * 360, ...sample(uv) };
  }

  function updateMarker(time) {
    if (!marker.visible) return;
    const { uv } = marker.userData;
    const lon = (uv.x - 0.5) * 2 * Math.PI, lat = (uv.y - 0.5) * Math.PI;
    const d = new THREE.Vector3(Math.cos(lat) * Math.sin(lon), Math.sin(lat), Math.cos(lat) * Math.cos(lon));
    const flat = new THREE.Vector3((lon / Math.PI) * 1.6, (lat / Math.PI) * 1.6, 0.006);
    marker.position.copy(d.clone().multiplyScalar(1.016).lerp(flat, state.morph));
    marker.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), d.clone().lerp(new THREE.Vector3(0, 0, 1), state.morph).normalize());
    marker.scale.setScalar(0.028 * (1 + 0.12 * Math.sin(time * 3)));
  }

  function animate(dt, spinRate) {
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
    if (state.layerT < 1) {
      state.layerT = Math.min(1, state.layerT + dt / 0.45);
      surfaceMat.uniforms.uLayerMix.value = ease(state.layerT);
    }
  }

  function frame(dt, simTime, spinRate) {
    animate(dt, spinRate);
    shared.uTime.value = simTime;
    shared.uMorph.value = state.morph;
    surfaceMat.uniforms.uRot.value.setFromMatrix4(group.matrixWorld);
    atmoMat.uniforms.uOpacity.value = 1 - Math.min(1, state.morph * 2.5);
    atmosphere.visible = atmoMat.uniforms.uOpacity.value > 0.001;
    clouds.visible = cloudMat.uniforms.uOpacity.value > 0;
    shared.uSun.value.copy(SUN_VIEW).applyQuaternion(camera.quaternion);
    updateMarker(performance.now() / 1000);
    controls.update();
    renderer.render(scene, camera);
  }

  return {
    renderer, bake, resize, setMode, setLayer, pick, frame,
    setClouds: (on) => { cloudMat.uniforms.uOpacity.value = on ? 1 : 0; },
    setWorld: (p) => { Object.entries(p).forEach(([k, v]) => { bakeMat.uniforms[k].value = v; }); bake(); },
    landFraction: () => estimateLand(renderer, probeRT),
  };
}

function starSprite() {
  const c = document.createElement('canvas');
  c.width = c.height = 32;
  const g = c.getContext('2d');
  const grad = g.createRadialGradient(16, 16, 0, 16, 16, 16);
  grad.addColorStop(0, 'rgba(255,255,255,1)');
  grad.addColorStop(0.35, 'rgba(255,255,255,0.35)');
  grad.addColorStop(1, 'rgba(255,255,255,0)');
  g.fillStyle = grad;
  g.fillRect(0, 0, 32, 32);
  return new THREE.CanvasTexture(c);
}

function makeStars() {
  const n = 2200, pos = new Float32Array(n * 3), col = new Float32Array(n * 3);
  for (let i = 0; i < n; i++) {
    const u = Math.random() * 2 - 1, a = Math.random() * Math.PI * 2, s = Math.sqrt(1 - u * u);
    pos.set([s * Math.cos(a) * 120, u * 120, s * Math.sin(a) * 120], i * 3);
    const b = Math.pow(Math.random(), 10) * 1.6 + 0.0025, warm = Math.random();
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

function makeMarker() {
  const m = new THREE.Mesh(
    new THREE.RingGeometry(0.62, 1, 48),
    new THREE.MeshBasicMaterial({ color: 0xf4ede2, transparent: true, opacity: 0.95, depthWrite: false, side: THREE.DoubleSide }),
  );
  m.visible = false;
  m.renderOrder = 2;
  return m;
}

function estimateLand(renderer, rt) {
  const buf = new Uint16Array(rt.width * rt.height * 4);
  renderer.readRenderTargetPixels(rt, 0, 0, rt.width, rt.height, buf);
  let land = 0, total = 0;
  for (let j = 0; j < rt.height; j++) {
    const wgt = Math.cos(((j + 0.5) / rt.height - 0.5) * Math.PI);
    for (let i = 0; i < rt.width; i++) {
      total += wgt;
      if (THREE.DataUtils.fromHalfFloat(buf[(j * rt.width + i) * 4]) > 0) land += wgt;
    }
  }
  return land / total;
}
