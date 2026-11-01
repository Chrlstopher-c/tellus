// Écran de départ : monde généré ou corps réel, puis époque de départ.
import { ArrowRight, Dices } from 'lucide-react';
import { useState, type ReactElement } from 'react';
import { BODIES, type WorldConfig } from './world-config';

interface Generated {
  seed: number;
  water: number;
  size: number;
  distance: number;
}

interface SliderProps {
  label: string;
  unit: string;
  min: number;
  max: number;
  value: number;
  onChange(v: number): void;
}

function Slider(p: SliderProps): ReactElement {
  return (
    <label className="field">
      <span>{p.label}<em>{p.value} {p.unit}</em></span>
      <input type="range" min={p.min} max={p.max} value={p.value} onChange={(e) => p.onChange(Number(e.target.value))} />
    </label>
  );
}

function GeneratedPane({ g, set }: { g: Generated; set(g: Generated): void }): ReactElement {
  return (
    <div className="start-pane">
      <label className="field"><span>Graine</span>
        <div className="seed">
          <input value={g.seed} inputMode="numeric" onChange={(e) => set({ ...g, seed: Number(e.target.value) || 0 })} />
          <button title="Graine aléatoire" onClick={() => set({ ...g, seed: Math.floor(Math.random() * 99999) })}><Dices /></button>
        </div>
      </label>
      <Slider label="Océans" unit="%" min={40} max={95} value={g.water} onChange={(water) => set({ ...g, water })} />
      <Slider label="Taille" unit="% Terre" min={50} max={160} value={g.size} onChange={(size) => set({ ...g, size })} />
      <Slider label="Distance à l’étoile" unit="% UA" min={70} max={150} value={g.distance}
        onChange={(distance) => set({ ...g, distance })} />
    </div>
  );
}

const orb = (c: string): string =>
  `radial-gradient(circle at 35% 30%, color-mix(in srgb, ${c} 75%, white), ${c} 45%, color-mix(in srgb, ${c} 30%, black) 100%)`;

function BodiesPane({ body, set }: { body: string; set(name: string): void }): ReactElement {
  return (
    <div className="start-pane"><div className="bodies">
      {BODIES.map((b) => (
        <button key={b.name} className={`body${b.name === body ? ' active' : ''}`} onClick={() => set(b.name)}>
          <span className="orb" style={{ background: orb(b.color) }} />
          <span><b>{b.name}</b><small>{b.facts}</small></span>
        </button>
      ))}
    </div></div>
  );
}

function Epochs({ mature, set }: { mature: boolean; set(m: boolean): void }): ReactElement {
  return (
    <div className="start-epoch"><p className="panel-label">Départ</p>
      <div className="epochs">
        <button className={`epoch${mature ? '' : ' active'}`} onClick={() => set(false)}>
          <b>Planète jeune</b><small>Océans formés, aucune vie. Tout reste à émerger.</small></button>
        <button className={`epoch${mature ? ' active' : ''}`} onClick={() => set(true)}>
          <b>Laisser mûrir</b><small>Calcul accéléré jusqu’à <em>4 Ga</em> avant d’ouvrir les yeux.</small></button>
      </div>
    </div>
  );
}

function toConfig(tab: 'gen' | 'sol', g: Generated, body: string, mature: boolean): WorldConfig {
  const source: WorldConfig['source'] = tab === 'sol' ? { kind: 'body', name: body }
    : { kind: 'generated', seed: g.seed, waterPercent: g.water, sizePercent: g.size, distancePercent: g.distance };
  return { source, matureToYears: mature ? 4e9 : null };
}

export function StartScreen({ onLaunch }: { onLaunch(c: WorldConfig): void }): ReactElement {
  const [tab, setTab] = useState<'gen' | 'sol'>('gen');
  const [g, setG] = useState<Generated>({ seed: 48213, water: 71, size: 100, distance: 100 });
  const [body, setBody] = useState('Terre');
  const [mature, setMature] = useState(false);
  return (
    <section className="start" data-testid="start">
      <div className="start-card glass">
        <p className="wordmark">Tellus</p>
        <h2 className="start-title">Un monde, des milliards d’années.</h2>
        <p className="start-lead">Choisissez une planète. Les lois de la physique et du vivant feront le reste.</p>
        <div className="start-tabs">
          <button className={`start-tab${tab === 'gen' ? ' active' : ''}`} onClick={() => setTab('gen')}>Monde généré</button>
          <button className={`start-tab${tab === 'sol' ? ' active' : ''}`} onClick={() => setTab('sol')}>Système solaire</button>
        </div>
        {tab === 'gen' ? <GeneratedPane g={g} set={setG} /> : <BodiesPane body={body} set={setBody} />}
        <Epochs mature={mature} set={setMature} />
        <button className="launch" data-testid="launch" onClick={() => onLaunch(toConfig(tab, g, body, mature))}>
          Créer le monde <ArrowRight />
        </button>
      </div>
    </section>
  );
}
