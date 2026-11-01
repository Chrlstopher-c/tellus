// Calques de la vue : nom, pastille et légende, générés depuis les palettes du shader (mêmes couleurs garanties).
import { PAL, cssHex, cssRamp, type Rgb } from './shaders/palettes';

export interface LegendBar {
  kind: 'bar';
  background: string;
  scale: readonly string[];
}

export interface LegendCats {
  kind: 'cats';
  items: readonly { color: string; name: string }[];
}

export interface LegendNote {
  kind: 'note';
  text: string;
}

export interface Layer {
  name: string;
  swatch: string;
  legend: LegendBar | LegendCats | LegendNote;
}

const swatch = (arr: Rgb[]): string => cssRamp(arr).replace('90deg', '135deg');

export const LAYERS: readonly Layer[] = [
  { name: 'Naturel', swatch: 'linear-gradient(135deg,#2a3f24,#c9b07c 55%,#0d2a4d)',
    legend: { kind: 'note', text: 'Couleurs vraies, dérivées du relief, du climat et du vivant simulés.' } },
  { name: 'Relief', swatch: swatch(PAL.relief),
    legend: { kind: 'bar', background: cssRamp([PAL.reliefSea[1] ?? [0, 0, 0], PAL.reliefSea[0] ?? [0, 0, 0], ...PAL.relief]),
      scale: ['−5,5 km', '+5,2 km'] } },
  { name: 'Température', swatch: swatch(PAL.temp),
    legend: { kind: 'bar', background: cssRamp(PAL.temp), scale: ['−30 °C', '+35 °C'] } },
  { name: 'Précipitations', swatch: swatch(PAL.rain),
    legend: { kind: 'bar', background: cssRamp(PAL.rain), scale: ['25', '320', '4 000 mm/an'] } },
  { name: 'Biomes', swatch: `conic-gradient(${PAL.biomes.map((b) => cssHex(b[1])).join(',')})`,
    legend: { kind: 'cats', items: PAL.biomes.map(([name, c]) => ({ name, color: cssHex(c) })) } },
  { name: 'Productivité', swatch: swatch(PAL.life),
    legend: { kind: 'bar', background: cssRamp(PAL.life), scale: ['0', '1 100', '2 200 gC/m²/an'] } },
];
