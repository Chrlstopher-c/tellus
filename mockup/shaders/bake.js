// Cuisson des champs physiques dans une texture équirectangulaire : R altitude (m), G température (°C),
// B précipitations (mm/an), A variation locale 0..1. Globe, carte, nuages et sonde lisent tous cette texture.
import { NOISE } from './noise.js';

export const BAKE_VERT = /* glsl */ `
varying vec2 vUv;
void main(){ vUv=uv; gl_Position=vec4(position.xy,0.0,1.0); }
`;

export const BAKE_FRAG = /* glsl */ `
precision highp float;
varying vec2 vUv;
uniform float uSeed;
uniform float uSea;
uniform float uTempOffset;
uniform float uWet;
${NOISE}

float elevationAt(vec3 d,out float landOut,out float mountOut){
  vec3 s=vec3(uSeed*0.137,uSeed*0.071,-uSeed*0.113);
  vec3 q=d*1.25+s;
  vec3 w=vec3(fbm(q+vec3(3.1,0.0,0.0),4),fbm(q+vec3(0.0,7.7,0.0),4),fbm(q+vec3(0.0,0.0,11.3),4));
  float cont=fbm(q*0.95+w*0.8,7);
  float land=cont-uSea;
  vec3 r=d*2.6+s*1.7+w*1.2;
  float mount=ridged(r);
  float belt=smoothstep(0.35,0.75,fbm(d*1.8+s*2.3,3)*0.5+0.5);
  float detail=fbm(d*22.0+q,4);
  float elev;
  if(land>0.0){
    float inland=smoothstep(0.0,0.22,land);
    elev=40.0+land*2400.0+mount*mount*5200.0*inland*(0.35+0.65*belt)+detail*180.0*inland;
  }else{
    float shelf=smoothstep(0.0,0.06,-land);
    elev=-120.0-shelf*3100.0-(-land)*2600.0+detail*120.0+mount*900.0*shelf*(1.0-belt);
  }
  landOut=land;
  mountOut=mount*belt;
  return elev;
}

void main(){
  vec3 d=dirFromUv(vUv);
  float land,mount;
  float elev=elevationAt(d,land,mount);
  float lat=asin(d.y);
  float L=abs(lat)*180.0/PI;
  float n1=fbm(d*5.0+vec3(uSeed),3);
  float t=29.0-49.0*pow(abs(sin(lat)),1.3)+uTempOffset;
  t-=6.5*max(elev,0.0)/1000.0;
  t+=n1*2.5;
  if(elev<0.0)t=max(t,-1.8+min(0.0,t+18.0)*0.4);
  float p=2500.0*exp(-pow(L/9.5,2.0))+1150.0*exp(-pow((L-52.0)/13.0,2.0))+260.0;
  p*=1.0-0.72*exp(-pow((L-26.0)/8.5,2.0));
  if(land>0.0){
    p*=mix(1.0,0.38,smoothstep(0.04,0.32,land));
    p+=mount*1100.0*smoothstep(0.0,0.1,land);
  }
  p*=0.72+0.56*(fbm(d*3.3-vec3(uSeed),4)*0.5+0.5);
  p*=uWet;
  p=max(p,25.0);
  float variation=fbm(d*9.0+vec3(2.0,uSeed,5.0),4)*0.5+0.5;
  gl_FragColor=vec4(elev,t,p,variation);
}
`;
