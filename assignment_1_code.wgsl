fn np(x: f32, y: f32) -> vec2f {
  return vec2f(x, y) * vec2f(1., res.y/res.x);
}

fn swirl(pos: vec2f, p1: vec2f, p2: vec2f, offset1: f32, offset2: f32) -> f32 {
  let cent = np(0.5, 1.);
  let offs1 = sin(length(pos-cent) * 5. * PI2)*offset1;
  let offs2 = sin(length(pos-cent) * 5. * PI2)*offset2;
  let d1 = 1.-clamp(abs(10.*dot(pos-cent, cross(vec3f(normalize(p1-cent+offs1+offs2), 0.), vec3f(0., 0., 1.)).xy)), 0., 1.);
  let d2 = 1.-clamp(abs(10.*dot(pos-cent, cross(vec3f(normalize(p2-cent+offs1+offs2), 0.), vec3f(0., 0., 1.)).xy)), 0., 1.);
  return d1+d2;
}

fn perpdot(p1: vec2f, p2: vec2f) -> f32 {
  return p1.x * p2.y - p1.y * p2.x;
}

fn line(pos: vec2f, p1: vec2f, p2: vec2f, sharp: f32, wavem: f32, wavef: f32) -> f32 {
  let offset = wavem*sin(wavef*length(pos-p1));
  return sigmoid(perpdot(normalize(pos-p1), normalize(p2-p1)) + offset, sharp);
}

fn sigmoid(n: f32, sharp: f32) -> f32 {
  return (1./(1.+exp(-sharp*n)));
}

@fragment 
fn fs( @builtin(position) pos : vec4f ) -> @location(0) vec4f {
  let npos = np(pos.x/res.x, pos.y/res.y);

  let volume = length(audio);
  var t = 20.;
  let offset = 5. * volume;

  let ah = audio.x;
  let am = audio.y;
  let al = 0.2*audio.z;

  let pivot = vec2f(0.7, 0.2);

  let base = np(0.5, 0.5);

  let hf = line(npos, pivot, base + vec2f(al, 0.), offset+t, am, ah * 10.);
  let mf = line(npos, pivot, base, offset+t, am, ah * 10.);
  let lf = line(npos, pivot, base - vec2f(al, 0.), offset+t, am, ah * 10.);
  var rots1 = 0.;
  rots1 = al;
  var rots2 = 0.;
  rots2 = al;
  var squares = 0.;
  squares = 1.;
  var lsquares = 0.;
  lsquares = 1.;
  var bg = vec4f(hf, mf, lf, 1.);
  let tpos = vec2f(npos.x % 0.5 * 2., npos.y);
  let ro = mat2x2(cos(rots1*1.5), -sin(rots1*1.5), sin(rots1*1.5), cos(rots1*1.5));
  let nro = mat2x2(cos(-rots2*0.5), -sin(-rots2*.5), sin(-rots2*.5), cos(-rots2*.5));
  let swirlieness = al;
  var d = swirl((npos*nro*10.) % 1., np(1., 1.), np(0.5, 0.0), swirlieness, 0.0)*squares;
  var dbb = swirl(((npos*ro/.5*20.) % 1.), np(0.5, 2.), np(1.0, 1.0), lf*5., al) * smoothstep(0.6, 0.8, volume)*0.1 * lsquares;
  d = clamp(volume-d, 0., 1.);
  bg = 1.-bg; d=1.-d;
  return vec4f(d*bg.x + dbb*(1.-lf), d*bg.y + dbb*(1.-lf), d*bg.z + dbb*(1.-lf), 1.);
}
