// Lecture défensive du détail de région renvoyé en JSON par la simulation.
import type { RegionDetail, Role, SpeciesLine } from '../inspector/region';

const num = (o: unknown, k: string): number => {
  const v = typeof o === 'object' && o !== null ? Reflect.get(o, k) : undefined;
  return typeof v === 'number' && Number.isFinite(v) ? v : 0;
};
const field = (o: unknown, k: string): unknown => (typeof o === 'object' && o !== null ? Reflect.get(o, k) : undefined);
const ROLES: readonly Role[] = ['producer', 'herbivore', 'predator'];

function species(o: unknown): SpeciesLine {
  const role = field(o, 'role');
  const trend = field(o, 'trend');
  return {
    id: num(o, 'id'), name: String(field(o, 'name') ?? ''), population: num(o, 'population'),
    role: ROLES.find((r) => r === role) ?? 'producer',
    trend: Array.isArray(trend) ? trend.filter((x): x is number => typeof x === 'number') : [],
  };
}

export function parseRegion(json: string, lat: number, lon: number): RegionDetail {
  const o: unknown = JSON.parse(json);
  const soil = field(o, 'soil');
  const list = field(o, 'species');
  const biome = field(o, 'biome');
  return {
    lat, lon, elevation: num(o, 'elevation'), temperature: num(o, 'temperature'), precipitation: num(o, 'precipitation'),
    iceFraction: num(o, 'iceFraction'), vegCover: num(o, 'vegCover'), nppGcM2Yr: num(o, 'nppGcM2Yr'),
    soil: { water: num(soil, 'water'), carbon: num(soil, 'carbon'), nitrogen: num(soil, 'nitrogen'), phosphorus: num(soil, 'phosphorus') },
    biome: typeof biome === 'string' && biome.length > 0 ? biome : null,
    speciesCount: num(o, 'speciesCount'),
    species: Array.isArray(list) ? list.slice(0, 6).map(species) : [],
  };
}
