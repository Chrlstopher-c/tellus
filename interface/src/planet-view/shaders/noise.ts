// Bruit simplex 3D (Ashima Arts / Stefan Gustavson, MIT) + fBm, partagé par tous les shaders.
export const NOISE = /* glsl */ `
vec3 mod289(vec3 x){return x-floor(x*(1.0/289.0))*289.0;}
vec4 mod289(vec4 x){return x-floor(x*(1.0/289.0))*289.0;}
vec4 permute(vec4 x){return mod289(((x*34.0)+10.0)*x);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159-0.85373472095314*r;}
float snoise(vec3 v){
  const vec2 C=vec2(1.0/6.0,1.0/3.0);
  const vec4 D=vec4(0.0,0.5,1.0,2.0);
  vec3 i=floor(v+dot(v,C.yyy));
  vec3 x0=v-i+dot(i,C.xxx);
  vec3 g=step(x0.yzx,x0.xyz);
  vec3 l=1.0-g;
  vec3 i1=min(g.xyz,l.zxy);
  vec3 i2=max(g.xyz,l.zxy);
  vec3 x1=x0-i1+C.xxx;
  vec3 x2=x0-i2+C.yyy;
  vec3 x3=x0-D.yyy;
  i=mod289(i);
  vec4 p=permute(permute(permute(i.z+vec4(0.0,i1.z,i2.z,1.0))+i.y+vec4(0.0,i1.y,i2.y,1.0))+i.x+vec4(0.0,i1.x,i2.x,1.0));
  float n_=0.142857142857;
  vec3 ns=n_*D.wyz-D.xzx;
  vec4 j=p-49.0*floor(p*ns.z*ns.z);
  vec4 x_=floor(j*ns.z);
  vec4 y_=floor(j-7.0*x_);
  vec4 x=x_*ns.x+ns.yyyy;
  vec4 y=y_*ns.x+ns.yyyy;
  vec4 h=1.0-abs(x)-abs(y);
  vec4 b0=vec4(x.xy,y.xy);
  vec4 b1=vec4(x.zw,y.zw);
  vec4 s0=floor(b0)*2.0+1.0;
  vec4 s1=floor(b1)*2.0+1.0;
  vec4 sh=-step(h,vec4(0.0));
  vec4 a0=b0.xzyw+s0.xzyw*sh.xxyy;
  vec4 a1=b1.xzyw+s1.xzyw*sh.zzww;
  vec3 p0=vec3(a0.xy,h.x);
  vec3 p1=vec3(a0.zw,h.y);
  vec3 p2=vec3(a1.xy,h.z);
  vec3 p3=vec3(a1.zw,h.w);
  vec4 norm=taylorInvSqrt(vec4(dot(p0,p0),dot(p1,p1),dot(p2,p2),dot(p3,p3)));
  p0*=norm.x;p1*=norm.y;p2*=norm.z;p3*=norm.w;
  vec4 m=max(0.5-vec4(dot(x0,x0),dot(x1,x1),dot(x2,x2),dot(x3,x3)),0.0);
  m=m*m;
  return 105.0*dot(m*m,vec4(dot(p0,x0),dot(p1,x1),dot(p2,x2),dot(p3,x3)));
}
float fbm(vec3 p,int oct){
  float a=0.5,s=0.0;
  for(int i=0;i<8;i++){
    if(i>=oct)break;
    s+=a*snoise(p);
    p=p*2.03+vec3(1.7,9.2,3.1);
    a*=0.5;
  }
  return s;
}
float ridged(vec3 p){
  float a=0.5,s=0.0,w=1.0;
  for(int i=0;i<6;i++){
    float n=1.0-abs(snoise(p));
    n*=n;n*=w;
    w=clamp(n*1.8,0.0,1.0);
    s+=n*a;
    p=p*2.11+vec3(4.1,1.3,7.9);
    a*=0.5;
  }
  return s;
}
const float PI=3.14159265359;
const float TAU=6.28318530718;
vec3 dirFromUv(vec2 uv){
  float lon=(uv.x-0.5)*TAU;
  float lat=(uv.y-0.5)*PI;
  return vec3(cos(lat)*sin(lon),sin(lat),cos(lat)*cos(lon));
}
vec2 uvFromDir(vec3 d){
  return vec2(atan(d.x,d.z)/TAU+0.5,asin(clamp(d.y,-1.0,1.0))/PI+0.5);
}
`;

// Chaîne tonale commune : décodage sRGB exact et épaule douce qui conserve la teinte.
export const TONE = /* glsl */ `
vec3 lin(vec3 c){return mix(c/12.92,pow((c+0.055)/1.055,vec3(2.4)),step(0.04045,c));}
vec3 shoulder(vec3 c){
  float m=max(max(c.r,c.g),c.b);
  return m<=0.8?c:c*(0.8+0.2*(1.0-exp(-(m-0.8)/0.2)))/m;
}
`;

// Morphing globe → carte partagé par la surface et les nuages (une seule règle).
export const MORPH = /* glsl */ `
uniform float uMorph;
varying vec2 vUv;
varying vec3 vDir;
varying vec3 vNormalW;
varying vec3 vPosW;
varying float vFlat;
void morphVertex(float radius,float zFlat){
  vUv=uv;
  float lon=(uv.x-0.5)*6.28318530718;
  float lat=(uv.y-0.5)*3.14159265359;
  vec3 d=vec3(cos(lat)*sin(lon),sin(lat),cos(lat)*cos(lon));
  vec3 flatPos=vec3(lon/3.14159265359*1.6,lat/3.14159265359*1.6,zFlat);
  float k=clamp(uMorph*1.45-(0.5-abs(uv.x-0.5))*0.9,0.0,1.0);
  k=k*k*(3.0-2.0*k);
  vFlat=k;
  vDir=d;
  vec4 wp=modelMatrix*vec4(mix(d*radius,flatPos,k),1.0);
  vPosW=wp.xyz;
  vNormalW=normalize(mix(normalize(mat3(modelMatrix)*d),vec3(0.0,0.0,1.0),k));
  gl_Position=projectionMatrix*viewMatrix*wp;
}
`;
