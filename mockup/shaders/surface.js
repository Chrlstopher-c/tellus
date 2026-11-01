// Surface : même maillage pour le globe et la carte (uMorph 0 → 1), couleurs dérivées des champs cuits.
import { NOISE } from './noise.js';
import { CLOUDS } from './clouds.js';

export const SURFACE_VERT = /* glsl */ `
uniform float uMorph;
varying vec2 vUv;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
varying float vFlat;
const float PI_V=3.14159265359;
void main(){
  vUv=uv;
  float lon=(uv.x-0.5)*6.28318530718;
  float lat=(uv.y-0.5)*PI_V;
  vec3 d=vec3(cos(lat)*sin(lon),sin(lat),cos(lat)*cos(lon));
  vec3 flatPos=vec3(lon/PI_V*1.6,lat/PI_V*1.6,0.0);
  float k=clamp(uMorph*1.45-(0.5-abs(uv.x-0.5))*0.9,0.0,1.0);
  k=k*k*(3.0-2.0*k);
  vFlat=k;
  vDir=d;
  vec3 pos=mix(d,flatPos,k);
  vec4 wp=modelMatrix*vec4(pos,1.0);
  vPosW=wp.xyz;
  vNormalW=normalize(mix(normalize(mat3(modelMatrix)*d),vec3(0.0,0.0,1.0),k));
  gl_Position=projectionMatrix*viewMatrix*wp;
}
`;

export const SURFACE_FRAG = /* glsl */ `
precision highp float;
uniform sampler2D uFields;
uniform vec2 uTexel;
uniform vec3 uSun;
uniform float uMorph;
uniform float uTime;
uniform int uLayer;
uniform float uCloudsOn;
uniform mat3 uRot;
varying vec2 vUv;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
varying float vFlat;
${NOISE}
${CLOUDS}

vec3 lin(vec3 c){return pow(c,vec3(2.2));}
vec3 ramp4(float t,vec3 a,vec3 b,vec3 c,vec3 d){
  t=clamp(t,0.0,1.0)*3.0;
  if(t<1.0)return mix(a,b,t);
  if(t<2.0)return mix(b,c,t-1.0);
  return mix(c,d,t-2.0);
}
vec3 ramp6(float t,vec3 a,vec3 b,vec3 c,vec3 d,vec3 e,vec3 f){
  t=clamp(t,0.0,1.0)*5.0;
  if(t<1.0)return mix(a,b,t);
  if(t<2.0)return mix(b,c,t-1.0);
  if(t<3.0)return mix(c,d,t-2.0);
  if(t<4.0)return mix(d,e,t-3.0);
  return mix(e,f,t-4.0);
}
float npp(float t,float p){
  return min(3000.0*(1.0-exp(-0.000664*p)),3000.0/(1.0+exp(1.315-0.119*t)));
}
vec3 oceanColor(float depth,float t){
  vec3 c=mix(lin(vec3(0.075,0.30,0.40)),lin(vec3(0.012,0.055,0.13)),smoothstep(0.0,3800.0,depth));
  c=mix(c,lin(vec3(0.16,0.46,0.50)),(1.0-smoothstep(25.0,220.0,depth))*0.55);
  return mix(c,lin(vec3(0.86,0.90,0.93)),1.0-smoothstep(-6.0,-1.6,t));
}
vec3 naturalLand(float e,float t,float p,float v){
  float I=p/max(t+10.0,1.0);
  vec3 sand=mix(lin(vec3(0.85,0.75,0.56)),lin(vec3(0.74,0.55,0.36)),v);
  vec3 steppe=lin(vec3(0.66,0.61,0.41));
  vec3 grass=mix(lin(vec3(0.49,0.53,0.29)),lin(vec3(0.42,0.48,0.25)),v);
  vec3 boreal=lin(vec3(0.17,0.24,0.18));
  vec3 temperate=lin(vec3(0.24,0.35,0.18));
  vec3 tropical=lin(vec3(0.13,0.25,0.11));
  vec3 forest=mix(boreal,temperate,smoothstep(2.0,11.0,t));
  forest=mix(forest,tropical,smoothstep(18.0,25.0,t));
  forest*=0.85+0.3*v;
  vec3 dry=mix(sand,steppe,smoothstep(7.0,17.0,I));
  vec3 wet=mix(grass,forest,smoothstep(28.0,52.0,I));
  vec3 c=mix(dry,wet,smoothstep(15.0,30.0,I));
  c=mix(lin(vec3(0.52,0.50,0.42)),c,smoothstep(-7.0,1.0,t));
  c=mix(c,lin(vec3(0.43,0.40,0.36)),smoothstep(2400.0,3800.0,e)*0.75);
  return mix(c,lin(vec3(0.93,0.95,0.97)),1.0-smoothstep(-12.0,-8.0,t));
}
vec3 biomeColor(float e,float t,float p){
  if(t<-9.0)return lin(vec3(0.90,0.93,0.96));
  if(t<-1.0)return lin(vec3(0.62,0.66,0.62));
  float I=p/max(t+10.0,1.0);
  if(t<6.0)return p>300.0?lin(vec3(0.27,0.43,0.38)):lin(vec3(0.62,0.66,0.62));
  if(t<19.0){
    if(I<12.0)return lin(vec3(0.80,0.70,0.48));
    if(I<30.0)return lin(vec3(0.68,0.72,0.42));
    return lin(vec3(0.32,0.56,0.33));
  }
  if(I<10.0)return lin(vec3(0.89,0.73,0.45));
  if(I<35.0)return lin(vec3(0.80,0.66,0.34));
  if(I<65.0)return lin(vec3(0.42,0.62,0.30));
  return lin(vec3(0.15,0.45,0.27));
}
vec3 layerColor(vec4 f){
  float e=f.r,t=f.g,p=f.b,v=f.a;
  bool sea=e<0.0;
  if(uLayer==0)return sea?oceanColor(-e,t):naturalLand(e,t,p,v);
  if(uLayer==1){
    if(sea)return mix(lin(vec3(0.33,0.55,0.70)),lin(vec3(0.05,0.12,0.27)),smoothstep(0.0,5500.0,-e));
    return ramp6(e/5200.0,lin(vec3(0.30,0.45,0.30)),lin(vec3(0.58,0.62,0.38)),lin(vec3(0.80,0.71,0.47)),
      lin(vec3(0.67,0.50,0.36)),lin(vec3(0.55,0.47,0.44)),lin(vec3(0.96,0.95,0.93)));
  }
  if(uLayer==2)return ramp6((t+30.0)/65.0,lin(vec3(0.17,0.26,0.47)),lin(vec3(0.37,0.56,0.71)),
    lin(vec3(0.83,0.88,0.87)),lin(vec3(0.95,0.78,0.54)),lin(vec3(0.85,0.40,0.25)),lin(vec3(0.52,0.14,0.16)));
  if(uLayer==3)return ramp6(log(p/25.0)/log(160.0),lin(vec3(0.33,0.24,0.15)),lin(vec3(0.71,0.60,0.38)),
    lin(vec3(0.86,0.85,0.71)),lin(vec3(0.44,0.69,0.69)),lin(vec3(0.18,0.43,0.62)),lin(vec3(0.10,0.22,0.46)));
  if(sea)return lin(vec3(0.035,0.06,0.09));
  if(uLayer==4)return biomeColor(e,t,p);
  return ramp4(npp(t,p)/2200.0,lin(vec3(0.10,0.11,0.10)),lin(vec3(0.20,0.32,0.20)),
    lin(vec3(0.44,0.64,0.35)),lin(vec3(0.82,0.91,0.62)));
}
vec3 bumpNormal(vec3 n,float e){
  if(e<=0.0)return n;
  float ex=texture2D(uFields,vUv+vec2(uTexel.x,0.0)).r-texture2D(uFields,vUv-vec2(uTexel.x,0.0)).r;
  float ey=texture2D(uFields,vUv+vec2(0.0,uTexel.y)).r-texture2D(uFields,vUv-vec2(0.0,uTexel.y)).r;
  float lon=(vUv.x-0.5)*TAU,lat=(vUv.y-0.5)*PI;
  vec3 east=uRot*vec3(cos(lon),0.0,-sin(lon));
  vec3 north=uRot*vec3(-sin(lat)*sin(lon),cos(lat),-sin(lat)*cos(lon));
  east=normalize(mix(east,vec3(1.0,0.0,0.0),vFlat));
  north=normalize(mix(north,vec3(0.0,1.0,0.0),vFlat));
  float k=0.00042/max(cos(lat),0.25);
  return normalize(n-(east*ex*k+north*ey*k*0.5));
}
vec3 aces(vec3 x){return clamp((x*(2.51*x+0.03))/(x*(2.43*x+0.59)+0.14),0.0,1.0);}

void main(){
  vec4 f=texture2D(uFields,vUv);
  vec3 base=layerColor(f);
  vec3 N=normalize(vNormalW);
  vec3 Nb=bumpNormal(N,f.r);
  vec3 V=normalize(cameraPosition-vPosW);
  vec3 L=normalize(uSun);
  float ndl=dot(Nb,L);
  float day=smoothstep(-0.12,0.32,dot(N,L));
  float diff=max(ndl,0.0)*0.92+0.08*day;
  vec3 sunCol=mix(vec3(1.0,0.62,0.38),vec3(1.0,0.97,0.92),smoothstep(0.0,0.35,dot(N,L)));
  vec3 lit=base*diff*sunCol*2.2;
  if(f.r<0.0&&uLayer==0){
    vec3 H=normalize(L+V);
    float wob=0.82+0.18*snoise(vDir*180.0+uTime*0.05);
    float spec=pow(max(dot(N,H),0.0),140.0*wob)*1.6+pow(max(dot(N,H),0.0),16.0)*0.05;
    lit+=vec3(1.0,0.93,0.82)*spec*day*(1.0-f.g*0.0);
  }
  float cd=cloudDensity(vDir,uTime,f.b);
  lit*=1.0-0.38*cd*uCloudsOn*day*(1.0-vFlat);
  float fres=pow(1.0-max(dot(N,V),0.0),2.6);
  vec3 sky=vec3(0.32,0.55,1.0);
  lit=mix(lit,sky*0.55*day,fres*0.6*(1.0-vFlat));
  lit+=base*vec3(0.010,0.014,0.024)*(1.0-day);
  float hill=clamp(0.62+0.55*dot(Nb,normalize(vec3(-0.6,0.7,0.9))),0.3,1.25);
  vec3 flatLit=base*hill*1.35;
  flatLit=mix(flatLit,vec3(0.95),cd*uCloudsOn*0.85);
  vec3 col=mix(lit,flatLit,vFlat);
  col=aces(col*1.05);
  gl_FragColor=vec4(col,1.0);
  #include <colorspace_fragment>
}
`;
