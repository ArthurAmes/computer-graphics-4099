struct Star {
    position: vec3<f32>,
    mass: f32,
    velocity: vec3<f32>,
    bright: f32
}

@group(0) @binding(0)
var<storage, read_write> stars: array<Star>;
@group(0) @binding(1)
var<uniform> n_stars: u32;

const G: f32 = 6.67430E-11;
const S: f32 = 1.0E6;

const BLOCK_SIZE: i32 = 64;

var<workgroup> pos_shared: array<vec3f, BLOCK_SIZE>;
var<workgroup> mass_shared: array<f32, BLOCK_SIZE>;

@compute
@workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
    let block_idx = i32(lid.x);
    let S2 = S*S;

    var f = vec3f(0.0);
    let pos = stars[id.x].position;
    for(var i: i32 = 0; i < i32(n_stars); i+=BLOCK_SIZE) {
        pos_shared[block_idx] = stars[i+block_idx].position;
        mass_shared[block_idx] = stars[i+block_idx].mass;
        workgroupBarrier();

        for(var j: i32 = 0; j < BLOCK_SIZE; j++) {
            let idx = i + j;
            if idx != i32(id.x) {
                let v = pos_shared[j] - pos;
                // let r = length(v);
                // let dir = normalize(v);
                let r = pow(dot(v, v) + S2, 1.5); // almost doubles performance
                //f += dir * (G * mass_shared[j]) / (r * r + S2);
                f += ((G * mass_shared[j]) / r) * v;
            }
        }
    }

    stars[id.x].velocity += f;
    stars[id.x].position += stars[id.x].velocity;
}