// Palettes uniques : injectées dans le GLSL ET utilisées pour les légendes CSS (mêmes couleurs garanties).
// Valeurs en sRGB 0..1 ; le shader les linéarise avec lin().
export const PAL = {
  reliefSea: [[0.33, 0.55, 0.70], [0.05, 0.12, 0.27]],
  relief: [[0.30, 0.45, 0.30], [0.58, 0.62, 0.38], [0.80, 0.71, 0.47], [0.67, 0.50, 0.36], [0.55, 0.47, 0.44], [0.96, 0.95, 0.93]],
  temp: [[0.17, 0.26, 0.47], [0.37, 0.56, 0.71], [0.83, 0.88, 0.87], [0.95, 0.78, 0.54], [0.85, 0.40, 0.25], [0.52, 0.14, 0.16]],
  rain: [[0.33, 0.24, 0.15], [0.71, 0.60, 0.38], [0.86, 0.85, 0.71], [0.44, 0.69, 0.69], [0.18, 0.43, 0.62], [0.10, 0.22, 0.46]],
  life: [[0.10, 0.11, 0.10], [0.20, 0.32, 0.20], [0.44, 0.64, 0.35], [0.82, 0.91, 0.62]],
  biomes: [
    ['Inlandsis', [0.90, 0.93, 0.96]], ['Toundra', [0.62, 0.66, 0.62]], ['Taïga', [0.27, 0.43, 0.38]],
    ['Steppe froide', [0.80, 0.70, 0.48]], ['Prairie tempérée', [0.68, 0.72, 0.42]], ['Forêt tempérée', [0.32, 0.56, 0.33]],
    ['Désert chaud', [0.89, 0.73, 0.45]], ['Savane', [0.80, 0.66, 0.34]], ['Forêt saisonnière', [0.42, 0.62, 0.30]],
    ['Forêt tropicale humide', [0.15, 0.45, 0.27]],
  ],
};

const v3 = (c) => `vec3(${c.map((x) => x.toFixed(3)).join(',')})`;
const list = (name, arr) => `const vec3 ${name}[${arr.length}]=vec3[${arr.length}](${arr.map(v3).join(',')});`;

export const PALETTE_GLSL = [
  list('P_RELIEF_SEA', PAL.reliefSea), list('P_RELIEF', PAL.relief), list('P_TEMP', PAL.temp),
  list('P_RAIN', PAL.rain), list('P_LIFE', PAL.life), list('P_BIOME', PAL.biomes.map((b) => b[1])),
].join('\n');

export const cssHex = (c) => `#${c.map((x) => Math.round(x * 255).toString(16).padStart(2, '0')).join('')}`;
export const cssRamp = (arr) => `linear-gradient(90deg,${arr.map(cssHex).join(',')})`;
