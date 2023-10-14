@group(0) @binding(0) var<uniform> res:   vec2f;

@group(0) @binding(9) var<storage> stateA: array<f32>;
@group(0) @binding(11) var<storage> stateB: array<f32>;
@group(0) @binding(13) var<storage> delta: array<f32>;

fn index( x:f32, y:f32 ) -> u32 {
  let w = res.x;
  let h = res.y;
  return u32( (y % h) * w + (w*-.5) + (x % w) );
}

@fragment 
fn fs( @builtin(position) pos : vec4f ) -> @location(0) vec4f {
  let idx : u32 = index( pos.x, pos.y );
  let A = stateA[ idx ];
  let B = stateB[ idx ];
  let D = delta[ idx ];
  return vec4f( -D * 200., B, D * 200., 1.);
}