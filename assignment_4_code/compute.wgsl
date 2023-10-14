struct Particle {
  pos: vec2f,
  vel: vec2f
};

@group(0) @binding(0) var<uniform> frame: f32;
@group(0) @binding(1) var<uniform> res:   vec2f;
@group(0) @binding(2) var<uniform> ts:   f32;
@group(0) @binding(3) var<storage, read_write> state: array<Particle>;

fn cellindex( cell:vec3u ) -> u32 {
  let size = 8u;
  return cell.x + (cell.y * size) + (cell.z * size * size);
}

@compute
@workgroup_size(8,8)

fn cs(@builtin(global_invocation_id) cell:vec3u)  {
  let i = cellindex( cell );
  let p = state[ i ];
  var acc = vec2f(0.0, 0.0);
  const G = 6.67430E-11;
  for(var j: u32 = 0; j < 1024; j++) {
    if( i != j) {
      let pp = state[j].pos - p.pos;
      let l = length(pp);
      let d = normalize(pp);
      acc += d / ((l * l) + 0.00001);
    }
  }

  state[i].vel += G * acc * ts;
  state[i].pos += state[i].vel * ts;
}