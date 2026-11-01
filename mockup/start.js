// Écran de départ : choisir ou générer un monde, puis l'époque de départ.
const BODIES = [
  { name: 'Mercure', facts: '0,39 UA · sans atmosphère · 167 °C', color: '#a39c94' },
  { name: 'Vénus', facts: '0,72 UA · 92 bar · 464 °C', color: '#d9b77a' },
  { name: 'Terre', facts: '1,00 UA · 1 bar · 15 °C', color: '#5b8fb9' },
  { name: 'Lune', facts: '1,00 UA · sans atmosphère · −20 °C', color: '#b9b5ae' },
  { name: 'Mars', facts: '1,52 UA · 6 mbar · −63 °C', color: '#c46f4a' },
  { name: 'Titan', facts: '9,5 UA · 1,5 bar · −179 °C', color: '#c99a4e' },
];

export function renderStart(el, onLaunch) {
  el.innerHTML = `
    <div class="start-card glass">
      <p class="wordmark">Tellus</p>
      <h2 class="start-title">Un monde, des milliards d’années.</h2>
      <p class="start-lead">Choisissez une planète. Les lois de la physique et du vivant feront le reste.</p>
      <div class="start-tabs">
        <button class="start-tab active" data-tab="gen">Monde généré</button>
        <button class="start-tab" data-tab="sol">Système solaire</button>
      </div>
      <div class="start-pane" data-pane="gen">
        <label class="field"><span>Graine</span><div class="seed"><input id="seed" value="48213" inputmode="numeric" /><button id="dice" title="Graine aléatoire"><i data-lucide="dices"></i></button></div></label>
        ${slider('water', 'Océans', 40, 95, 71, '%')}
        ${slider('size', 'Taille', 50, 160, 100, '% Terre')}
        ${slider('sun', 'Distance à l’étoile', 70, 150, 100, '% UA')}
      </div>
      <div class="start-pane" data-pane="sol" hidden>
        <div class="bodies">${BODIES.map((b, i) => `<button class="body${i === 2 ? ' active' : ''}"><span class="orb" style="--c:${b.color}"></span><span><b>${b.name}</b><small>${b.facts}</small></span></button>`).join('')}</div>
      </div>
      <div class="start-epoch">
        <p class="panel-label">Départ</p>
        <div class="epochs">
          <button class="epoch active"><b>Planète jeune</b><small>Océans formés, aucune vie. Tout reste à émerger.</small></button>
          <button class="epoch"><b>Laisser mûrir</b><small>Calcul accéléré jusqu’à <em>4 Ga</em> avant d’ouvrir les yeux.</small></button>
        </div>
      </div>
      <button class="launch" id="launch">Créer le monde <i data-lucide="arrow-right"></i></button>
    </div>`;
  wire(el, onLaunch);
}

function slider(id, label, min, max, val, unit) {
  return `<label class="field"><span>${label}<em id="${id}-v">${val} ${unit}</em></span><input type="range" id="${id}" min="${min}" max="${max}" value="${val}" data-unit="${unit}" /></label>`;
}

function wire(el, onLaunch) {
  el.querySelectorAll('.start-tab').forEach((t) => t.addEventListener('click', () => {
    el.querySelectorAll('.start-tab').forEach((x) => x.classList.toggle('active', x === t));
    el.querySelectorAll('.start-pane').forEach((p) => { p.hidden = p.dataset.pane !== t.dataset.tab; });
  }));
  el.querySelectorAll('input[type=range]').forEach((r) => r.addEventListener('input', () => {
    el.querySelector(`#${r.id}-v`).textContent = `${r.value} ${r.dataset.unit}`;
  }));
  const pickOne = (sel) => el.querySelectorAll(sel).forEach((b) => b.addEventListener('click', () => {
    el.querySelectorAll(sel).forEach((x) => x.classList.toggle('active', x === b));
  }));
  pickOne('.body');
  pickOne('.epoch');
  el.querySelector('#dice').addEventListener('click', () => { el.querySelector('#seed').value = Math.floor(Math.random() * 99999); });
  el.querySelector('#launch').addEventListener('click', () => {
    el.classList.add('leaving');
    setTimeout(() => { el.hidden = true; onLaunch(); }, 700);
  });
}
