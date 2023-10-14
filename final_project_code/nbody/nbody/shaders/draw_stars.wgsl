@group(0) @binding(0)
var<uniform> vp_mat: mat4x4<f32>;

const UNIVERSE_SIZE: f32 = 9.0E8;

struct Star {
    @location(0) position: vec3<f32>,
    @location(1) mass: f32,
    @location(2) velocity: vec3<f32>,
    @location(3) col: f32
}

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) mass: f32
}

@fragment
fn fs_main(vo: VertexOut) -> @location(0) vec4<f32> {
    return vec4f(
        smoothstep(0.0, 0.33, vo.mass), 
        smoothstep(0.33, 0.66, vo.mass),
        smoothstep(0.66, 1.0, vo.mass),
        1.0
    );
}

@vertex
fn vs_main(
    star: Star
) -> VertexOut {
    let p = vp_mat * vec4f(star.position/vec3f(UNIVERSE_SIZE/2.0) - vec3f(1.0, 1.0, 1.0), 1.0);

    return VertexOut(p, star.col);
}