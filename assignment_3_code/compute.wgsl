@group(0) @binding(0) var<uniform> res: vec2f;
@group(0) @binding(1) var<uniform> mouse: vec3f;
@group(0) @binding(2) var<uniform> DA: f32;
@group(0) @binding(3) var<uniform> DB: f32;
@group(0) @binding(4) var<uniform> F: f32;
@group(0) @binding(5) var<uniform> K: f32;
@group(0) @binding(6) var<uniform> DT: f32;
@group(0) @binding(7) var<uniform> Scale: f32;
@group(0) @binding(8) var<uniform> Mult: f32;


@group(0) @binding(9) var<storage, read_write> stateAin: array<f32>;
@group(0) @binding(10) var<storage, read_write> stateAout: array<f32>;

@group(0) @binding(11) var<storage, read_write> stateBin: array<f32>;
@group(0) @binding(12) var<storage, read_write> stateBout: array<f32>;

@group(0) @binding(13) var<storage, read_write> deltain: array<f32>;
@group(0) @binding(14) var<storage, read_write> deltaout: array<f32>;

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

fn index( x:i32, y:i32 ) -> u32 {
  let _res = vec2i(res);
  return u32( abs(y % _res.y) * _res.x + abs(x % _res.x ) );
}

@compute
@workgroup_size(8,8)
fn cs( @builtin(global_invocation_id) _cell:vec3u ) {
  let cell = vec3i(_cell);
  
  let i = index(cell.x, cell.y);
  
  let mxd = f32(cell.x) - mouse.x*res.x;
  let myd = f32(cell.y) - mouse.y*res.y;
  
  let A = stateAin[i];
  let B = stateBin[i];
  
  let lapA = (
    0.05 * stateAin[index(cell.x - 1, cell.y + 1)] +
    0.20 * stateAin[index(cell.x    , cell.y + 1)] +
    0.05 * stateAin[index(cell.x + 1, cell.y + 1)] +
    0.20 * stateAin[index(cell.x - 1, cell.y)]     +
    -1.0 * A +
    0.20 * stateAin[index(cell.x + 1, cell.y)]     +
    0.05 * stateAin[index(cell.x - 1, cell.y - 1)] +
    0.20 * stateAin[index(cell.x    , cell.y - 1)] +
    0.05 * stateAin[index(cell.x + 1, cell.y - 1)]
  );
  
  let lapB = (
    0.05 * stateBin[index(cell.x - 1, cell.y + 1)] +
    0.20 * stateBin[index(cell.x    , cell.y + 1)] +
    0.05 * stateBin[index(cell.x + 1, cell.y + 1)] +
    0.20 * stateBin[index(cell.x - 1, cell.y)]     +
    -1.0 * B +
    0.20 * stateBin[index(cell.x + 1, cell.y)]     +
    0.05 * stateBin[index(cell.x - 1, cell.y - 1)] +
    0.20 * stateBin[index(cell.x    , cell.y - 1)] +
    0.05 * stateBin[index(cell.x + 1, cell.y - 1)]
  );
  
  let n = noise(vec2f(f32(cell.x)/res.x, f32(cell.y)/res.y) * Scale * 10.0);
  
  let F = F + Mult * n;
  let K = K + Mult * n;
  
  stateAout[i] = A + (DA * lapA - A * B * B + F * (1 - A)) * DT;
  stateBout[i] = B + (DB * lapB + A * B * B - B * (K + F)) * DT;
  
  if((mxd*mxd + myd*myd) < 200.0 * mouse.z) {
    stateBout[i] = 1.0;
  }
  
  deltaout[i] = 0.9 * deltain[i] + 0.1 * (stateAout[i] - A);
}
