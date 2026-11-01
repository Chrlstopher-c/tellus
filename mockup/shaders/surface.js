// Surface : même maillage pour le globe et la carte (uMorph 0 → 1), couleurs dérivées des champs cuits.
import { NOISE, TONE, MORPH } from './noise.js';
import { PALETTE_GLSL } from './palettes.js';

export const SURFACE_VERT = /* glsl */ `
${MORPH}
void main(){ morphVertex(1.0,0.0); }
`;

export const SURFACE_FRAG = /* glsl */ `
precision highp float;
uniform sampler2D uFields;
uniform vec2 uTexel;
uniform vec3 uSun;
uniform float uTime;
uniform int uLayerA;
uniform int uLayerB;
uniform float uLayerMix;
uniform mat3 uRot;
varying vec2 vUv;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
varying float vFlat;
${NOISE}
${TONE}
${PALETTE_GLSL}

vec3 ramp6(float t,vec3 p[6]){
  t=clamp(t,0.0,1.0)*5.0;
  int i=int(min(floor(t),4.0));
  return mix(lin(p[i]),lin(p[i+1]),t-float(i));
}
vec3 ramp4(float t,vec3 p[4]){
  t=clamp(t,0.0,1.0)*3.0;
  int i=int(min(floor(t),2.0));
  return mix(lin(p[i]),lin(p[i+1]),t-float(i));
}
float npp(float t,float p){
  return min(3000.0*(1.0-exp(-0.000664*p)),3000.0/(1.0+exp(1.315-0.119*t)));
}
vec3 oceanColor(float depth,float t){
  vec3 c=mix(lin(vec3(0.05,0.16,0.30)),lin(vec3(0.02,0.07,0.17)),smoothstep(150.0,4000.0,depth));
  c=mix(c,lin(vec3(0.10,0.32,0.40)),(1.0-smoothstep(20.0,90.0,depth))*0.4);
  return mix(c,lin(vec3(0.86,0.90,0.93)),1.0-smoothstep(-6.0,-1.6,t));
}
vec3 naturalLand(float e,float t,float p,float v){
  float I=p/max(t+10.0,1.0);
  vec3 sand=mix(lin(vec3(0.85,0.75,0.56)),lin(vec3(0.74,0.55,0.36)),v);
  vec3 steppe=lin(vec3(0.66,0.61,0.41));
  vec3 grass=mix(lin(vec3(0.49,0.53,0.29)),lin(vec3(0.42,0.48,0.25)),v);
  vec3 forest=mix(lin(vec3(0.17,0.24,0.18)),lin(vec3(0.24,0.35,0.18)),smoothstep(2.0,11.0,t));
  forest=mix(forest,lin(vec3(0.14,0.25,0.12)),smoothstep(18.0,25.0,t));
  forest*=0.85+0.3*v;
  vec3 dry=mix(sand,steppe,smoothstep(7.0,17.0,I));
  vec3 wet=mix(grass,forest,smoothstep(28.0,52.0,I));
  vec3 c=mix(dry,wet,smoothstep(15.0,30.0,I));
  c=mix(lin(vec3(0.52,0.50,0.42)),c,smoothstep(-7.0,1.0,t));
  c=mix(c,lin(vec3(0.43,0.40,0.36)),smoothstep(2400.0,3800.0,e)*0.75);
  return mix(c,lin(vec3(0.93,0.95,0.97)),1.0-smoothstep(-12.0,-8.0,t));
}
int biomeIndex(float t,float p){
  if(t<-9.0)return 0;
  if(t<-1.0)return 1;
  float I=p/max(t+10.0,1.0);
  if(t<6.0)return p>300.0?2:1;
  if(t<19.0)return I<12.0?3:(I<30.0?4:5);
  return I<10.0?6:(I<35.0?7:(I<65.0?8:9));
}
vec3 landLayer(int layer,float e,float t,float p,float v){
  if(layer==0)return naturalLand(e,t,p,v);
  if(layer==1)return ramp6(e/5200.0,P_RELIEF);
  if(layer==2)return ramp6((t+30.0)/65.0,P_TEMP);
  if(layer==3)return ramp6(log(p/25.0)/log(160.0),P_RAIN);
  if(layer==4)return lin(P_BIOME[biomeIndex(t,p)]);
  return ramp4(npp(t,p)/2200.0,P_LIFE);
}
vec3 seaLayer(int layer,float e,float t,float p){
  if(layer==0)return oceanColor(max(-e,0.0),t);
  if(layer==1)return mix(lin(P_RELIEF_SEA[0]),lin(P_RELIEF_SEA[1]),smoothstep(0.0,5500.0,-e));
  if(layer==2)return ramp6((t+30.0)/65.0,P_TEMP);
  if(layer==3)return ramp6(log(p/25.0)/log(160.0),P_RAIN);
  return lin(vec3(0.035,0.06,0.09));
}
vec3 layerColor(vec4 f,int layer,float landW){
  return mix(seaLayer(layer,f.r,f.g,f.b),landLayer(layer,max(f.r,0.0),f.g,f.b,f.a),landW);
}
vec3 bumpNormal(vec3 n,float landW){
  float ex=texture2D(uFields,vUv+vec2(uTexel.x,0.0)).r-texture2D(uFields,vUv-vec2(uTexel.x,0.0)).r;
  float ey=texture2D(uFields,vUv+vec2(0.0,uTexel.y)).r-texture2D(uFields,vUv-vec2(0.0,uTexel.y)).r;
  float lon=(vUv.x-0.5)*TAU,lat=(vUv.y-0.5)*PI;
  vec3 east=normalize(mix(uRot*vec3(cos(lon),0.0,-sin(lon)),vec3(1.0,0.0,0.0),vFlat));
  vec3 north=normalize(mix(uRot*vec3(-sin(lat)*sin(lon),cos(lat),-sin(lat)*cos(lon)),vec3(0.0,1.0,0.0),vFlat));
  float k=0.00042/max(cos(lat),0.25)*landW;
  return normalize(n-(east*ex*k+north*ey*k*0.5));
}

void main(){
  vec4 f=texture2D(uFields,vUv);
  float w=max(fwidth(f.r),1.0);
  float landW=smoothstep(-w,w,f.r);
  vec3 base=mix(layerColor(f,uLayerA,landW),layerColor(f,uLayerB,landW),uLayerMix);
  vec3 N=normalize(vNormalW);
  vec3 Nb=bumpNormal(N,landW);
  vec3 V=normalize(cameraPosition-vPosW);
  vec3 L=normalize(uSun);
  float mu=dot(N,L);
  float day=smoothstep(-0.12,0.32,mu);
  vec3 sunCol=mix(vec3(1.0,0.72,0.52),vec3(1.0,0.98,0.95),smoothstep(0.0,0.12,mu));
  vec3 lit=base*(max(dot(Nb,L),0.0)*0.92+0.08*day)*sunCol*1.5;
  float natural=uLayerA==0||uLayerB==0?(uLayerA==0&&uLayerB==0?1.0:(uLayerA==0?1.0-uLayerMix:uLayerMix)):0.0;
  vec3 H=normalize(L+V);
  float wob=0.82+0.18*snoise(vDir*180.0+uTime*0.05);
  float spec=pow(max(dot(N,H),0.0),140.0*wob)*1.6+pow(max(dot(N,H),0.0),16.0)*0.05;
  float ice=1.0-smoothstep(-6.0,-1.6,f.g);
  lit+=vec3(1.0,0.95,0.86)*spec*day*(1.0-ice)*(1.0-landW)*natural;
  float fres=pow(1.0-max(dot(N,V),0.0),2.6);
  float a=fres*(1.0-vFlat);
  lit=lit*(1.0-0.35*a)+vec3(0.20,0.42,1.0)*0.55*smoothstep(-0.35,0.25,mu)*a;
  lit+=base*vec3(0.010,0.014,0.024)*(1.0-day);
  float hill=clamp(0.62+0.55*dot(Nb,normalize(vec3(-0.6,0.7,0.9))),0.3,1.25);
  vec3 flatLit=base*hill/1.005;
  gl_FragColor=vec4(shoulder(mix(lit,flatLit,vFlat)),1.0);
  #include <colorspace_fragment>
}
`;
