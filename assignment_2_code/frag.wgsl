@group(0) @binding(0) var<uniform> resolution: vec2f;
@group(0) @binding(1) var<uniform> frame: f32;
@group(0) @binding(2) var<uniform> mouse: vec3f;
@group(0) @binding(3) var<uniform> octaves: f32;
@group(0) @binding(4) var<uniform> scale: f32;
@group(0) @binding(5) var<uniform> speed: f32;
@group(0) @binding(6) var backSampler:    sampler;
@group(0) @binding(7) var backBuffer:     texture_2d<f32>;
@group(0) @binding(8) var videoSampler:   sampler;
@group(1) @binding(0) var videoBuffer:    texture_external;

fn grad( z: vec2f ) -> vec2f {
  // 2D to 1D  (feel free to replace by some other)
  var n = i32(z.x+z.y*11111);

  // Hugo Elias hash (feel free to replace by another one)
  n = (n<<13)^n;
  n = (n*(n*n*15731+789221)+1376312589)>>16;
  
  return vec2(cos(f32(n)),sin(f32(n)));
}

fn noise(_st: vec2f) -> f32 {
  let i = floor(_st);
  let f = fract(_st);
  
  let u = f*f*f*(f*(f*6.0 - 15.0)+10.);
  
  return mix( mix(   dot( grad( i+vec2f(0, 0) ), f-vec2f(0.0, 0.0) ), 
                     dot( grad( i+vec2f(1, 0) ), f-vec2f(1.0, 0.0) ), u.x),
              mix(   dot( grad( i+vec2f(0, 1) ), f-vec2f(0.0, 1.0) ), 
                     dot( grad( i+vec2f(1, 1) ), f-vec2f(1.0, 1.0) ), u.x), u.y);
}

fn nn(_st: vec2f, oct: i32) -> f32 {
  var a = 1.;
  var o = 0.0;
  var m = 10.;
  for(var i: i32 = 0; i < oct; i++ ) {
    o += noise(_st * m) * a;
    a = a / 2.0;
    m = m * 2;
  }
  return o;
}

@fragment 
fn fs( @builtin(position) pos : vec4f ) -> @location(0) vec4f {
  let t = frame / 10000.;
  let p = pos.xy / resolution;
  
  let grad_x = (-0.5 * length(textureSampleBaseClampToEdge(videoBuffer, videoSampler, vec2f(p.x - 1.0/resolution.x, p.y))) + 
                0.5 * length(textureSampleBaseClampToEdge(videoBuffer, videoSampler, vec2f(p.x + 1.0/resolution.x, p.y))));
                
  let grad_y = (-0.5 * length(textureSampleBaseClampToEdge(videoBuffer, videoSampler, vec2f(p.x, p.y - 1.0/resolution.y))) + 
                0.5 * length(textureSampleBaseClampToEdge(videoBuffer, videoSampler, vec2f(p.x, p.y + 1.0/resolution.y))));
                
  let grad = vec2f(grad_x, grad_y);

  let step = 1/resolution;
  
  let sp = p * scale;
  let oct = i32(floor(octaves));
  
  var curl = vec2f(-0.5 * nn(vec2f(sp.x - step.x, sp.y), oct) + 0.5 * nn(vec2f(sp.x + step.x, sp.y), oct),
                     -0.5 * nn(vec2f(sp.x, sp.y - step.y ), oct) + 0.5 * nn(vec2f(sp.x, sp.y + step.y ), oct));
                     
  curl = vec2f(curl.y, -curl.x) * speed;
  var bb = textureSample(backBuffer, backSampler, p + curl);
  bb.a = 1 - mouse.z;
  if(frame == 0.0) {
    bb.a = 1.0;
  } 
  
  return vec4f(vec3f( (bb.a) * textureSampleBaseClampToEdge(videoBuffer, videoSampler, p).xyz + (1.-bb.a) * bb.xyz), 0.0);
}

