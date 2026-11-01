// Passe de résolution : champs par cellule de la simulation → texture équirectangulaire lue par le globe et la carte.
// R altitude (m), G température (°C), B précipitations (mm/an), A couvert végétal 0..1 (vivant).
// Deux instantanés (précédent, suivant) sont interpolés par uAlpha : le rendu reste fluide entre deux ticks.
// Le champ brut, linéaire par triangle, est ensuite lissé (SMOOTH_FRAG) puis enrichi du détail sous la maille.
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
  gl_FragColor=f;
}
`;

/** Flou gaussien séparable (~une demi-maille L6) ; la passe finale ajoute le détail sous la maille. */
export const SMOOTH_FRAG = /* glsl */ `
precision highp float;
varying vec2 vUv;
uniform sampler2D uSrc;
uniform vec2 uStep;
uniform float uFinal;
uniform float uSeed;
${NOISE}
void main(){
  float W[5]=float[5](0.2270,0.1946,0.1216,0.0541,0.0162);
  vec4 c=texture2D(uSrc,vUv)*W[0];
  for(int i=1;i<5;i++){
    vec2 o=uStep*float(i)*3.0;
    c+=(texture2D(uSrc,vUv+o)+texture2D(uSrc,vUv-o))*W[i];
  }
  if(uFinal>0.5){
    vec3 d=dirFromUv(vUv);
    float keep=smoothstep(60.0,400.0,abs(c.r));
    c.r+=fbm(d*28.0+vec3(uSeed),4)*140.0*keep*(c.r>0.0?1.0:0.6);
    c.r+=fbm(d*24.0-vec3(uSeed),3)*50.0*(1.0-keep);
    c.a=clamp(c.a,0.0,1.0);
  }
  gl_FragColor=c;
}
`;
