// Signes vitaux de la planète, lus dans l'instantané de la simulation (jamais inventés).
import type { ReactElement } from 'react';

export interface PlanetVitals {
  o2Percent: number;
  co2Ppm: number;
  meanTempC: number;
  species: number;
  biomassGtC: number;
}

const fmt = (n: number, d = 0): string =>
  n.toLocaleString('fr-FR', { minimumFractionDigits: d, maximumFractionDigits: d });

export function Vitals({ v }: { v: PlanetVitals }): ReactElement {
  const items: readonly [string, string, string][] = [
    ['Oxygène', fmt(v.o2Percent, v.o2Percent < 1 ? 2 : 1), '%'],
    ['CO₂', fmt(v.co2Ppm), 'ppm'],
    ['Temp. moyenne', fmt(v.meanTempC, 1), '°C'],
    ['Espèces', fmt(v.species), ''],
    ['Biomasse', fmt(v.biomassGtC), 'GtC'],
  ];
  return (
    <section className="vitals glass" data-testid="vitals">
      {items.map(([name, value, unit]) => (
        <div className="vital" key={name}><p>{name}</p><p>{value}<small>{unit}</small></p></div>
      ))}
    </section>
  );
}
