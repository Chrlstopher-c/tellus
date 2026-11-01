// Passe de résolution : champs par cellule de la simulation → texture équirectangulaire lue par le globe et la carte.
// R altitude (m), G température (°C), B précipitations (mm/an), A couvert végétal 0..1 (vivant).
// Deux instantanés (précédent, suivant) sont interpolés par uAlpha : le rendu reste fluide entre deux ticks.
// Le détail sous la maille ne change jamais le signe de l'altitude (une côte simulée reste une côte).
import { NOISE } from './noise';

export const RESOLVE_VERT = /* glsl */ `
varying vec2 vUv;
void main(){ vUv=uv; gl_Position=vec4(position.xy,0.0,1.0); }
`;

export const RESOLVE_FRAG = /* glsl */ `
precision highp float;
precision highp sampler2D;
varying vec2 vUv;
uniform sampler2D uLookupIds;
uniform sampler2D uLookupWeights;
uniform sampler2D uCellsPrev;
uniform sampler2D uCellsNext;
uniform float uAlpha;
uniform int uCellsWidth;
uniform float uSeed;
${NOISE}

vec4 cellAt(float id){
  int i=int(id+0.5);
  ivec2 c=ivec2(i-(i/uCellsWidth)*uCellsWidth,i/uCellsWidth);
  return mix(texelFetch(uCellsPrev,c,0),texelFetch(uCellsNext,c,0),uAlpha);
}

void main(){
  ivec2 t=ivec2(gl_FragCoord.xy);
  vec3 ids=texelFetch(uLookupIds,t,0).xyz;
  vec2 w=texelFetch(uLookupWeights,t,0).xy;
  vec4 f=cellAt(ids.x)*w.x+cellAt(ids.y)*w.y+cellAt(ids.z)*(1.0-w.x-w.y);
  vec3 d=dirFromUv(vUv);
  float detail=fbm(d*28.0+vec3(uSeed),4);
  float keep=smoothstep(60.0,400.0,abs(f.r));
  f.r+=detail*140.0*keep*(f.r>0.0?1.0:0.6);
  f.a=clamp(f.a,0.0,1.0);
  gl_FragColor=f;
}
`;
