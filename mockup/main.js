// Point d'entrée de la maquette : relie la scène 3D à l'interface.
import { createPlanet } from './planet.js';
import { renderStart } from './start.js';
import {
  renderLayers, renderInspector, renderVitals, renderTimeline, renderSpeeds, renderPowers,
  renderWorldMenu, formatYear, SPEEDS, WORLDS,
} from './ui.js';

const $ = (id) => document.getElementById(id);
const planet = createPlanet($('scene'));
const sim = { year: 4093412806, speed: 2, playing: true, visualTime: 0, world: 0, power: null };

planet.resize();
planet.bake();
window.addEventListener('resize', planet.resize);

renderLayers($('layer-list'), $('legend'), planet.setLayer);
renderVitals($('vitals'));
renderTimeline($('ticks'));
renderSpeeds($('speeds'), sim.speed, (i) => { sim.speed = i; updateSpeedLabel(); });
renderPowers($('powers'), (p) => {
  sim.power = p;
  document.body.classList.toggle('targeting', !!p);
  $('power-hint').hidden = !p;
  if (p) $('power-hint').textContent = `${p.label} — cliquez sur la planète pour cibler`;
});
$('clouds').addEventListener('change', (e) => planet.setClouds(e.target.checked));

function updateSpeedLabel() {
  $('speed-label').textContent = sim.playing ? SPEEDS[sim.speed].desc : 'en pause';
}
updateSpeedLabel();

$('play').addEventListener('click', () => {
  sim.playing = !sim.playing;
  $('play').innerHTML = `<i data-lucide="${sim.playing ? 'pause' : 'play'}"></i>`;
  window.lucide.createIcons();
  updateSpeedLabel();
});

const segs = document.querySelectorAll('.seg');
const pill = document.querySelector('.seg-pill');
function placePill(btn) { pill.style.width = `${btn.offsetWidth}px`; pill.style.transform = `translateX(${btn.offsetLeft - 4}px)`; }
segs.forEach((b) => b.addEventListener('click', () => {
  segs.forEach((x) => x.classList.toggle('active', x === b));
  placePill(b);
  planet.setMode(b.dataset.mode);
}));

$('dual').addEventListener('click', (e) => {
  const on = e.currentTarget.classList.toggle('on');
  toast(on ? 'Mode deux écrans activé' : 'Mode deux écrans désactivé',
    on ? 'La carte et les panneaux passent sur le second écran.' : 'Tout revient sur un seul écran.', 'var(--select)');
});

const menu = $('world-menu');
$('world-button').addEventListener('click', () => {
  menu.hidden = !menu.hidden;
  $('world-button').setAttribute('aria-expanded', String(!menu.hidden));
  renderWorldMenu(menu, sim.world, pickWorld);
});
function pickWorld(i) {
  const w = WORLDS[i];
  menu.hidden = true;
  $('world-button').setAttribute('aria-expanded', 'false');
  if (w.real) { toast(`${w.name} — bientôt`, 'Les planètes réelles arrivent avec le moteur de simulation (E2).', 'var(--climate)'); return; }
  sim.world = i;
  $('world-name').textContent = w.name;
  $('world-sub').textContent = w.sub;
  planet.setWorld(w.params);
  renderInspector($('inspector'), null);
}

let downAt = null;
$('scene').addEventListener('pointerdown', (e) => { downAt = [e.clientX, e.clientY]; });
$('scene').addEventListener('pointerup', (e) => {
  if (!downAt || Math.hypot(e.clientX - downAt[0], e.clientY - downAt[1]) > 4) return;
  const s = planet.pick(e.clientX, e.clientY);
  if (!s) return;
  if (sim.power) {
    toast(`${sim.power.label} déclenchée`, `${Math.abs(s.lat).toFixed(1)}° ${s.lat >= 0 ? 'N' : 'S'} — les conséquences se propagent.`, 'var(--god)');
    document.querySelector('.power.active')?.click();
  }
  renderInspector($('inspector'), s);
});
$('inspector').addEventListener('click', (e) => { if (e.target.closest('[data-close]')) renderInspector($('inspector'), null); });

let toastTimer;
function toast(title, body, color = 'var(--life)') {
  const t = $('toast');
  $('toast-title').textContent = title;
  $('toast-body').textContent = body;
  t.querySelector('.toast-dot').style.background = color;
  t.hidden = false;
  t.style.animation = 'none'; void t.offsetWidth; t.style.animation = '';
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { t.hidden = true; }, 5200);
}
renderStart($('start'), () => {
  document.body.classList.remove('starting');
  setTimeout(() => toast('Nouvelle espèce : Thalorella velox', 'Petit prédateur arboricole, forêts de l’hémisphère sud.'), 2500);
});

let last = performance.now();
function loop(now) {
  const dt = Math.min(0.05, (now - last) / 1000);
  last = now;
  const sp = SPEEDS[sim.speed];
  if (sim.playing) {
    sim.year += sp.perSec * dt;
    sim.visualTime += dt * (0.6 + sim.speed * 0.45);
  }
  $('year').textContent = formatYear(sim.year);
  planet.frame(dt, sim.visualTime, sim.playing ? sp.spin : 0.004);
  requestAnimationFrame(loop);
}

window.lucide.createIcons();
placePill(document.querySelector('.seg.active'));
renderInspector($('inspector'), null);
requestAnimationFrame(loop);
window.tellus = { planet, sim };
