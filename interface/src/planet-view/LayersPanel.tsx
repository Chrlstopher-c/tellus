// Panneau des calques : choix du calque, légende, nuages.
import type { ReactElement } from 'react';
import { LAYERS, type Layer } from './layers';

interface Props {
  active: number;
  clouds: boolean;
  onLayer(index: number): void;
  onClouds(on: boolean): void;
}

function Legend({ legend }: { legend: Layer['legend'] }): ReactElement {
  if (legend.kind === 'note') return <p className="note">{legend.text}</p>;
  if (legend.kind === 'cats') {
    return (
      <div className="cats">
        {legend.items.map((c) => <span key={c.name} className="cat"><i style={{ background: c.color }} />{c.name}</span>)}
      </div>
    );
  }
  return (
    <>
      <div className="bar" style={{ background: legend.background }} />
      <div className="scale">{legend.scale.map((s) => <span key={s}>{s}</span>)}</div>
    </>
  );
}

export function LayersPanel(p: Props): ReactElement {
  const layer = LAYERS[p.active] ?? LAYERS[0];
  return (
    <aside className="layers glass" data-testid="layers">
      <p className="panel-label">Calques</p>
      <ul>
        {LAYERS.map((l, i) => (
          <li key={l.name}>
            <button className={`layer${i === p.active ? ' active' : ''}`} onClick={() => p.onLayer(i)}>
              <span className="swatch" style={{ background: l.swatch }} />{l.name}
            </button>
          </li>
        ))}
      </ul>
      <div className="legend">{layer && <Legend legend={layer.legend} />}</div>
      <label className="toggle">
        <input type="checkbox" checked={p.clouds} onChange={(e) => p.onClouds(e.target.checked)} /><span />Nuages
      </label>
    </aside>
  );
}
