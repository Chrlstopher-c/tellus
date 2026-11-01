// Nuages : densité pilotée par les précipitations cuites (pluvieux = nuageux), advectée par bandes de vents
// zonaux en flow-map à deux phases (jamais de cisaillement cumulé). Même maillage morphé que la surface.
import { NOISE, TONE, MORPH } from './noise';

const CLOUD_FN = /* glsl */ `
vec3 rotY(vec3 p,float a){float c=cos(a),s=sin(a);return vec3(c*p.x+s*p.z,p.y,-s*p.x+c*p.z);}
float cloudAt(vec3 p,float t,float precip){
  vec3 q=p*1.7+vec3(0.0,0.0,t*0.004);
  vec3 w=vec3(fbm(q+vec3(1.3,0.0,0.0),3),fbm(q+vec3(0.0,4.1,0.0),3),fbm(q+vec3(0.0,0.0,8.2),3));
  float n=fbm(q*1.2+w*0.55,6)*0.5+0.5;
  n=n*0.8+0.2*(fbm(q*5.0+w,3)*0.5+0.5);
  float thr=mix(0.66,0.42,smoothstep(150.0,2600.0,precip));
  return smoothstep(thr,thr+0.3,n);
}
float cloudDensity(vec3 d,float t,float precip){
  float lat=asin(clamp(d.y,-1.0,1.0));
  float w=-cos(4.0*lat);
  const float P=40.0;
  float a=fract(t/P),b=fract(t/P+0.5);
  float ca=cloudAt(rotY(d,(a-0.5)*P*0.035*w),t,precip);
  float cb=cloudAt(rotY(d,(b-0.5)*P*0.035*w)+17.0,t,precip);
  return mix(ca,cb,abs(2.0*a-1.0));
}
`;

export const CLOUD_VERT = /* glsl */ `
${MORPH}
void main(){ morphVertex(1.012,0.003); }
`;

export const CLOUD_FRAG = /* glsl */ `
precision highp float;
uniform sampler2D uFields;
uniform vec3 uSun;
uniform float uTime;
uniform float uOpacity;
varying vec2 vUv;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
varying float vFlat;
${NOISE}
${TONE}
${CLOUD_FN}
void main(){
  float d=cloudDensity(vDir,uTime,texture2D(uFields,vUv).b);
  vec3 N=normalize(vNormalW);
  float mu=dot(N,normalize(uSun));
  float lk=smoothstep(-0.08,0.22,mu);
  vec3 tint=mix(vec3(1.0,0.72,0.52),vec3(1.0),smoothstep(0.0,0.12,mu));
  vec3 col=mix(vec3(0.006,0.008,0.014),tint*1.05,lk);
  col=mix(col,vec3(0.95),vFlat);
  vec3 V=normalize(cameraPosition-vPosW);
  float edge=mix(smoothstep(0.0,0.25,dot(N,V)),1.0,vFlat);
  float shade=mix(mix(0.35,1.0,lk),1.0,vFlat);
  gl_FragColor=vec4(shoulder(col),d*0.92*uOpacity*edge*shade);
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
${TONE}
void main(){
  vec3 N=normalize(vNormalW);
  vec3 V=normalize(cameraPosition-vPosW);
  float t=clamp(-dot(N,V)/uLimb,0.0,1.0);
  float glow=pow(t,3.0);
  float mu=dot(N,normalize(uSun));
  float day=smoothstep(-0.12,0.2,mu);
  vec3 col=mix(vec3(0.9,0.5,0.3),vec3(0.20,0.42,1.0),smoothstep(-0.04,0.12,mu));
  gl_FragColor=vec4(shoulder(col*glow*day*0.55*uOpacity),1.0);
  #include <colorspace_fragment>
}
`;
