// Nuages : densité pilotée par les précipitations cuites (pluvieux = nuageux) et advectée par bandes de vents
// zonaux (alizés vers l'ouest, vents d'ouest aux latitudes moyennes, vents d'est polaires).
import { NOISE } from './noise.js';

export const CLOUDS = /* glsl */ `
vec3 rotY(vec3 p,float a){float c=cos(a),s=sin(a);return vec3(c*p.x+s*p.z,p.y,-s*p.x+c*p.z);}
float cloudDensity(vec3 d,float t,float precip){
  float lat=asin(clamp(d.y,-1.0,1.0));
  float wind=-cos(4.0*lat);
  vec3 p=rotY(d,t*0.035*wind);
  vec3 q=p*2.6+vec3(0.0,0.0,t*0.004);
  vec3 w=vec3(fbm(q+vec3(1.3,0.0,0.0),3),fbm(q+vec3(0.0,4.1,0.0),3),fbm(q+vec3(0.0,0.0,8.2),3));
  float n=fbm(q*1.4+w*1.1,5)*0.5+0.5;
  float wet=smoothstep(150.0,2600.0,precip);
  float thr=mix(0.66,0.42,wet);
  return smoothstep(thr,thr+0.22,n);
}
`;

export const CLOUD_VERT = /* glsl */ `
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
void main(){
  vDir=normalize(position);
  vNormalW=normalize(mat3(modelMatrix)*vDir);
  vec4 wp=modelMatrix*vec4(position,1.0);
  vPosW=wp.xyz;
  gl_Position=projectionMatrix*viewMatrix*wp;
}
`;

export const CLOUD_FRAG = /* glsl */ `
precision highp float;
uniform sampler2D uFields;
uniform vec3 uSun;
uniform float uTime;
uniform float uOpacity;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
${NOISE}
${CLOUDS}
void main(){
  float precip=texture2D(uFields,uvFromDir(vDir)).b;
  float d=cloudDensity(vDir,uTime,precip);
  vec3 N=normalize(vNormalW);
  float mu=dot(N,normalize(uSun));
  float day=smoothstep(-0.15,0.35,mu);
  vec3 col=mix(vec3(1.0,0.66,0.46),vec3(1.0),smoothstep(0.0,0.4,mu))*(0.18+0.95*day);
  col+=vec3(0.02,0.03,0.05);
  vec3 V=normalize(cameraPosition-vPosW);
  float edge=smoothstep(0.0,0.25,dot(N,V));
  gl_FragColor=vec4(col,d*0.92*uOpacity*edge);
  #include <colorspace_fragment>
}
`;

export const ATMO_VERT = /* glsl */ `
varying vec3 vNormalW;
varying vec3 vPosW;
void main(){
  vNormalW=normalize(mat3(modelMatrix)*normal);
  vec4 wp=modelMatrix*vec4(position,1.0);
  vPosW=wp.xyz;
  gl_Position=projectionMatrix*viewMatrix*wp;
}
`;

export const ATMO_FRAG = /* glsl */ `
precision highp float;
uniform vec3 uSun;
uniform float uOpacity;
uniform float uLimb;
varying vec3 vNormalW;
varying vec3 vPosW;
void main(){
  vec3 N=normalize(vNormalW);
  vec3 V=normalize(cameraPosition-vPosW);
  float t=clamp(-dot(N,V)/uLimb,0.0,1.0);
  float glow=pow(t,2.4);
  float mu=dot(N,normalize(uSun));
  float day=smoothstep(-0.35,0.25,mu);
  vec3 blue=vec3(0.30,0.56,1.0);
  vec3 dusk=vec3(1.0,0.48,0.22);
  vec3 col=mix(dusk,blue,smoothstep(-0.05,0.35,mu));
  gl_FragColor=vec4(col*glow*day*1.6*uOpacity,1.0);
  #include <colorspace_fragment>
}
`;
