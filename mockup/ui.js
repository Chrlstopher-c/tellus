// Interface de la maquette : calques, légendes, inspecteur, temps, pouvoirs, choix du monde.
export const LAYERS = [
  { id: 'natural', name: 'Naturel', swatch: 'linear-gradient(135deg,#2c4a2a,#c9b07c 60%,#0d3550)' },
  { id: 'relief', name: 'Relief', swatch: 'linear-gradient(135deg,#4c6b45,#c29c6a,#f0eee9)' },
  { id: 'temp', name: 'Température', swatch: 'linear-gradient(135deg,#2b4c7e,#cfe0e3,#d9663f)' },
  { id: 'rain', name: 'Précipitations', swatch: 'linear-gradient(135deg,#5a4026,#d8d6b5,#1b3f7a)' },
  { id: 'biome', name: 'Biomes', swatch: 'conic-gradient(#2a7345,#e0b973,#a8b26c,#6b8e86,#2a7345)' },
  { id: 'life', name: 'Productivité', swatch: 'linear-gradient(135deg,#1a1c1a,#4a7a3d,#d0e8a0)' },
];

const LEGENDS = [
  { note: 'Couleurs vraies, dérivées du climat et du relief simulés.' },
  { bar: 'linear-gradient(90deg,#0d1f45,#557fa6,#4c6b45,#a3a36b,#c29c6a,#8c7a6f,#f5f3ee)', scale: ['−6 km', '0', '+5 km'] },
  { bar: 'linear-gradient(90deg,#2b4c7e,#5f8fb4,#cfe0e3,#f2c58a,#d9663f,#8e2a2a)', scale: ['−30 °C', '0', '+35 °C'] },
  { bar: 'linear-gradient(90deg,#5a4026,#b59a62,#d8d6b5,#6fb0b0,#2d6e9e,#1b3f7a)', scale: ['25', '600', '4 000 mm/an'] },
  { cats: [['#e6edf4', 'Inlandsis'], ['#a0a8a0', 'Toundra'], ['#4a7064', 'Taïga'], ['#cfb47c', 'Steppe froide'],
    ['#acb56c', 'Prairie tempérée'], ['#55905a', 'Forêt tempérée'], ['#e3bb76', 'Désert chaud'], ['#cda75c', 'Savane'],
    ['#6c9f52', 'Forêt saisonnière'], ['#2a7345', 'Forêt tropicale humide']] },
  { bar: 'linear-gradient(90deg,#1a1c1a,#33522f,#73a35b,#d0e8a0)', scale: ['0', '1 100', '2 200 gC/m²/an'] },
];

export const WORLDS = [
  { name: 'Ascalon', sub: 'Monde généré · graine 48 213', note: 'Graine aléatoire, lois terrestres',
    params: { uSeed: 48.213, uSea: 0.11, uTempOffset: 0, uWet: 1 } },
  { name: 'Veyra', sub: 'Monde généré · graine 7 702', note: 'Planète océan, plus chaude',
    params: { uSeed: 7.702, uSea: 0.2, uTempOffset: 4, uWet: 1.25 } },
  { name: 'Orme', sub: 'Monde généré · graine 91 040', note: 'Supercontinent froid et sec',
    params: { uSeed: 91.04, uSea: -0.1, uTempOffset: -9, uWet: 0.65 } },
  { sep: true },
  { name: 'Terre', sub: 'Système solaire · 1,00 UA', note: 'Données NASA · 1 bar · 15 °C', real: true },
  { name: 'Mars', sub: 'Système solaire · 1,52 UA', note: 'Données NASA · 6 mbar · −63 °C', real: true },
  { name: 'Vénus', sub: 'Système solaire · 0,72 UA', note: 'Données NASA · 92 bar · 464 °C', real: true },
];

export const SPEEDS = [
  { label: '1 h/s', perSec: 1 / 8766, desc: 'une heure par seconde', spin: 0.26 },
  { label: '1 an/s', perSec: 1, desc: 'une année par seconde', spin: 0.05 },
  { label: '1 siècle/s', perSec: 100, desc: 'un siècle par seconde', spin: 0.035 },
  { label: '10 ka/s', perSec: 1e4, desc: 'dix mille ans par seconde', spin: 0.03 },
  { label: '1 Ma/s', perSec: 1e6, desc: 'un million d’années par seconde', spin: 0.03 },
];

const EVENTS = [
  { at: 0.03, color: 'var(--geo)', label: 'Premiers océans' },
  { at: 0.19, color: 'var(--life)', label: 'Apparition de la vie — dorsale australe' },
  { at: 0.42, color: 'var(--life)', label: 'Photosynthèse oxygénique' },
  { at: 0.51, color: 'var(--climate)', label: 'Glaciation globale' },
  { at: 0.71, color: 'var(--life)', label: 'Premiers organismes multicellulaires' },
  { at: 0.83, color: 'var(--life)', label: 'Sortie des eaux' },
  { at: 0.88, color: 'var(--god)', label: 'Intervention : météorite (Ø 9 km)' },
  { at: 0.905, color: 'var(--life)', label: 'Grandes forêts' },
];

export const POWERS = [
  { id: 'meteor', icon: 'sparkle', label: 'Météorite' },
  { id: 'volcano', icon: 'mountain-snow', label: 'Éruption volcanique' },
  { id: 'climate', icon: 'thermometer-sun', label: 'Forcer le climat' },
  { id: 'species', icon: 'dna', label: 'Introduire une espèce' },
  { id: 'ice', icon: 'snowflake', label: 'Glaciation' },
  { id: 'drought', icon: 'sun-dim', label: 'Sécheresse' },
];

const fmt = (n, d = 0) => n.toLocaleString('fr-FR', { maximumFractionDigits: d, minimumFractionDigits: d });

export function renderLayers(list, legend, onPick) {
  list.innerHTML = LAYERS.map((l, i) =>
    `<li><button class="layer${i ? '' : ' active'}" data-i="${i}"><span class="swatch" style="background:${l.swatch}"></span>${l.name}</button></li>`).join('');
  const show = (i) => {
    const g = LEGENDS[i];
    if (g.cats) legend.innerHTML = `<div class="cats">${g.cats.map(([c, n]) => `<span class="cat"><i style="background:${c}"></i>${n}</span>`).join('')}</div>`;
    else if (g.bar) legend.innerHTML = `<div class="bar" style="background:${g.bar}"></div><div class="scale">${g.scale.map((s) => `<span>${s}</span>`).join('')}</div>`;
    else legend.innerHTML = `<p class="note">${g.note}</p>`;
  };
  list.addEventListener('click', (e) => {
    const b = e.target.closest('.layer');
    if (!b) return;
    list.querySelectorAll('.layer').forEach((x) => x.classList.toggle('active', x === b));
    show(+b.dataset.i);
    onPick(+b.dataset.i);
  });
  show(0);
}

export function biomeName(t, p, e) {
  if (e < 0) return t < -2 ? ['Banquise', 'Océan gelé en surface'] : e > -250 ? ['Plateau continental', 'Mer peu profonde'] : ['Océan ouvert', 'Plaine abyssale'];
  if (t < -9) return ['Inlandsis', 'Glace permanente'];
  if (t < -1) return ['Toundra', 'Sol gelé, végétation rase'];
  const I = p / Math.max(t + 10, 1);
  if (t < 6) return p > 300 ? ['Taïga', 'Forêt boréale de conifères'] : ['Toundra sèche', 'Steppe froide'];
  if (t < 19) return I < 12 ? ['Steppe froide', 'Herbes basses, sols secs'] : I < 30 ? ['Prairie tempérée', 'Graminées et bosquets'] : ['Forêt tempérée', 'Feuillus et sous-bois'];
  if (I < 10) return ['Désert chaud', 'Dunes et regs'];
  if (I < 35) return ['Savane', 'Herbes hautes, arbres épars'];
  if (I < 65) return ['Forêt saisonnière', 'Saison sèche marquée'];
  return ['Forêt tropicale humide', 'Canopée dense, pluies toute l’année'];
}

const SYL = ['vel', 'ora', 'thal', 'myr', 'cae', 'lun', 'dor', 'sev', 'ith', 'qua', 'ner', 'ulm', 'ast', 'phy', 'ron', 'elis'];
const ENDS = ['ia', 'us', 'ax', 'ella', 'odon', 'is', 'ora', 'ium'];
const EPI = ['grandis', 'australis', 'velox', 'umbra', 'silvestris', 'minor', 'aquatica', 'borealis', 'ferox', 'lenta'];
function rng(seed) { let s = seed >>> 0; return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296); }
function speciesName(r) {
  const g = SYL[Math.floor(r() * SYL.length)] + SYL[Math.floor(r() * SYL.length)] + ENDS[Math.floor(r() * ENDS.length)];
  return g[0].toUpperCase() + g.slice(1) + ' ' + EPI[Math.floor(r() * EPI.length)];
}
function spark(r, color) {
  let y = 13, pts = [];
  for (let i = 0; i < 24; i++) { y = Math.max(3, Math.min(23, y + (r() - 0.48) * 5)); pts.push(`${(i / 23) * 76},${y.toFixed(1)}`); }
  return `<svg viewBox="0 0 76 26"><polyline points="${pts.join(' ')}" fill="none" stroke="${color}" stroke-width="1.4" stroke-linejoin="round" stroke-linecap="round"/></svg>`;
}
function npp(t, p) { return Math.min(3000 * (1 - Math.exp(-0.000664 * p)), 3000 / (1 + Math.exp(1.315 - 0.119 * t))); }

export function renderInspector(el, s) {
  if (!s) { el.classList.add('hidden'); return; }
  const [biome, sub] = biomeName(s.temperature, s.precipitation, s.elevation);
  const r = rng(Math.floor((s.lat + 90) * 1000 + (s.lon + 180) * 7));
  const sea = s.elevation < 0;
  const prod = sea ? 120 + r() * 200 : npp(s.temperature, s.precipitation);
  const roles = [['producer', 'Producteur', 'var(--life)'], ['herbivore', 'Herbivore', 'var(--time)'], ['predator', 'Prédateur', 'var(--god)']];
  const count = s.temperature < -9 ? 2 : 5;
  const species = Array.from({ length: count }, (_, i) => {
    const role = roles[i < 2 ? 0 : i < 4 ? 1 : 2];
    const pop = Math.floor(10 ** (6 - i * 1.1 + r()));
    return `<div class="sp"><div><p class="sp-name">${speciesName(r)}</p><p class="sp-meta"><span class="role ${role[0]}">${role[1]}</span>${fmt(pop)} ind.</p></div>${spark(r, role[2])}</div>`;
  }).join('');
  const lat = `${fmt(Math.abs(s.lat), 1)}° ${s.lat >= 0 ? 'N' : 'S'}`, lon = `${fmt(Math.abs(s.lon), 1)}° ${s.lon >= 0 ? 'E' : 'O'}`;
  const bars = [['Eau du sol', sea ? 1 : Math.min(1, s.precipitation / 2500)], ['Carbone', Math.min(1, prod / 2400)],
    ['Azote', 0.25 + r() * 0.6], ['Phosphore', 0.15 + r() * 0.5]];
  el.innerHTML = `
    <div class="insp-head"><div><p class="panel-label">Région sélectionnée</p><p class="insp-coords">${lat} · ${lon}</p></div>
      <button class="insp-close" data-close><i data-lucide="x"></i></button></div>
    <h2 class="insp-biome">${biome}</h2><p class="insp-biome-sub">${sub}</p>
    <div class="stats">
      <div class="stat"><p>${sea ? 'Profondeur' : 'Altitude'}</p><p>${fmt(Math.abs(s.elevation))}<small>m</small></p></div>
      <div class="stat"><p>Température moy.</p><p>${fmt(s.temperature, 1)}<small>°C</small></p></div>
      <div class="stat"><p>Précipitations</p><p>${fmt(s.precipitation)}<small>mm/an</small></p></div>
      <div class="stat"><p>Productivité</p><p>${fmt(prod)}<small>gC/m²/an</small></p></div>
    </div>
    <div class="section-title"><p class="panel-label">Sol et nutriments</p></div>
    <div class="bars">${bars.map(([n, v]) => `<div class="bar-row"><span>${n}</span><span class="b"><i style="width:${(v * 100).toFixed(0)}%"></i></span><span>${fmt(v * 100)} %</span></div>`).join('')}</div>
    <div class="section-title"><p class="panel-label">Espèces présentes</p><p class="panel-label">${fmt(count * 37 + Math.floor(r() * 40))}</p></div>
    <div class="species">${species}</div>`;
  el.classList.remove('hidden');
  window.lucide?.createIcons();
}

export function renderVitals(el) {
  const v = [['Oxygène', '20,9', '%', '+0,1', 'up'], ['CO₂', '412', 'ppm', '−3', 'down'], ['Temp. moyenne', '14,8', '°C', '+0,2', 'up'],
    ['Espèces', '18 402', '', '+56', 'up'], ['Biomasse', '548', 'GtC', '+2', 'up']];
  el.innerHTML = v.map(([n, x, u, t, d]) => `<div class="vital"><p>${n}</p><p>${x}<small>${u}</small><span class="trend ${d}">${t}</span></p></div>`).join('');
}

export function renderTimeline(ticks) {
  ticks.innerHTML = EVENTS.map((e) => `<span class="tick" style="left:${e.at * 100}%;background:${e.color}" data-label="${e.label}"></span>`).join('');
}

export function renderSpeeds(el, active, onPick) {
  el.innerHTML = SPEEDS.map((s, i) => `<button class="speed${i === active ? ' active' : ''}" data-i="${i}">${s.label}</button>`).join('');
  el.onclick = (e) => {
    const b = e.target.closest('.speed');
    if (!b) return;
    el.querySelectorAll('.speed').forEach((x) => x.classList.toggle('active', x === b));
    onPick(+b.dataset.i);
  };
}

export function renderPowers(el, onPick) {
  el.innerHTML = POWERS.map((p) => `<button class="power" data-id="${p.id}" data-label="${p.label}"><i data-lucide="${p.icon}"></i></button>`).join('');
  el.onclick = (e) => {
    const b = e.target.closest('.power');
    if (!b) return;
    const on = !b.classList.contains('active');
    el.querySelectorAll('.power').forEach((x) => x.classList.toggle('active', on && x === b));
    onPick(on ? POWERS.find((p) => p.id === b.dataset.id) : null);
  };
}

export function renderWorldMenu(el, current, onPick) {
  el.innerHTML = '<p class="panel-label" style="padding:6px 10px 4px">Mondes</p>' + WORLDS.map((w, i) => w.sep
    ? '<div class="sep"></div><p class="panel-label" style="padding:4px 10px">Système solaire</p>'
    : `<button data-i="${i}" class="${i === current ? 'active' : ''}"><span>${w.name}</span><small>${w.note}</small></button>`).join('');
  el.onclick = (e) => {
    const b = e.target.closest('button[data-i]');
    if (b) onPick(+b.dataset.i);
  };
}

export function formatYear(y) { return fmt(Math.floor(y)); }
