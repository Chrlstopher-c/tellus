// Inspecteur d'une région : physique, sol, espèces présentes. Rien n'est affiché qui ne vienne de la simulation.
import { X } from 'lucide-react';
import type { ReactElement } from 'react';
import { ROLE_COLOR, ROLE_LABEL, surfaceName, type RegionDetail, type SpeciesLine } from './region';

const fmt = (n: number, d = 0): string =>
  n.toLocaleString('fr-FR', { minimumFractionDigits: d, maximumFractionDigits: d });

function Spark({ values, color }: { values: readonly number[]; color: string }): ReactElement | null {
  if (values.length < 2) return null;
  const lo = Math.min(...values), hi = Math.max(...values), span = hi - lo || 1;
  const pts = values.map((v, i) => `${(i / (values.length - 1)) * 76},${(23 - ((v - lo) / span) * 20).toFixed(1)}`);
  return (
    <svg viewBox="0 0 76 26"><polyline points={pts.join(' ')} fill="none" stroke={color} strokeWidth="1.4"
      strokeLinejoin="round" strokeLinecap="round" /></svg>
  );
}

function Species({ s }: { s: SpeciesLine }): ReactElement {
  return (
    <div className="sp">
      <div>
        <p className="sp-name">{s.name}</p>
        <p className="sp-meta"><span className={`role ${s.role}`}>{ROLE_LABEL[s.role]}</span>{fmt(s.population)} ind.</p>
      </div>
      <Spark values={s.trend} color={ROLE_COLOR[s.role]} />
    </div>
  );
}

function Stats({ r }: { r: RegionDetail }): ReactElement {
  const sea = r.elevation < 0;
  const cells: readonly [string, string, string][] = [
    [sea ? 'Profondeur' : 'Altitude', fmt(Math.abs(r.elevation)), 'm'],
    ['Température moy.', fmt(r.temperature, 1), '°C'],
    ['Précipitations', fmt(r.precipitation), 'mm/an'],
    ['Productivité', fmt(r.nppGcM2Yr), 'gC/m²/an'],
  ];
  return (
    <div className="stats">
      {cells.map(([n, v, u]) => <div className="stat" key={n}><p>{n}</p><p>{v}<small>{u}</small></p></div>)}
    </div>
  );
}

function Soil({ r }: { r: RegionDetail }): ReactElement {
  const bars: readonly [string, number][] = [
    ['Eau du sol', r.soil.water], ['Carbone', r.soil.carbon], ['Azote', r.soil.nitrogen], ['Phosphore', r.soil.phosphorus],
  ];
  return (
    <div className="bars">
      {bars.map(([n, v]) => (
        <div className="bar-row" key={n}><span>{n}</span>
          <span className="b"><i style={{ width: `${Math.round(v * 100)}%` }} /></span><span>{fmt(v * 100)} %</span>
        </div>
      ))}
    </div>
  );
}

function coords(r: RegionDetail): string {
  return `${fmt(Math.abs(r.lat), 1)}° ${r.lat >= 0 ? 'N' : 'S'} · ${fmt(Math.abs(r.lon), 1)}° ${r.lon >= 0 ? 'E' : 'O'}`;
}

export function Inspector({ region, onClose }: { region: RegionDetail | null; onClose(): void }): ReactElement {
  if (!region) return <aside className="inspector glass hidden" />;
  const [name, sub] = surfaceName(region);
  return (
    <aside className="inspector glass" data-testid="inspector">
      <div className="insp-head">
        <div><p className="panel-label">Région sélectionnée</p><p className="insp-coords">{coords(region)}</p></div>
        <button className="insp-close" onClick={onClose} aria-label="Fermer"><X /></button>
      </div>
      <h2 className="insp-biome">{name}</h2><p className="insp-biome-sub">{sub}</p>
      <Stats r={region} />
      <div className="section-title"><p className="panel-label">Sol et nutriments</p></div>
      <Soil r={region} />
      <div className="section-title">
        <p className="panel-label">Espèces présentes</p><p className="panel-label">{fmt(region.speciesCount)}</p>
      </div>
      {region.species.length > 0
        ? <div className="species">{region.species.map((s) => <Species key={s.id} s={s} />)}</div>
        : <p className="insp-biome-sub">Aucune vie détectée dans cette région.</p>}
    </aside>
  );
}
