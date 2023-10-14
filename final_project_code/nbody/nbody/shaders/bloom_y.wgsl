@group(0) @binding(0)
var ping: texture_2d<f32>;
@group(0) @binding(2)
var pong: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    var g_7 = array<f32, 7>(
        0.01562500000,
        0.09375000000,
        0.2343750000,
        0.3125000000,
        0.2343750000,
        0.09375000000,
        0.01562500000
    );

    var acc = vec4f(0.0);
    for(var i: i32 = -3; i < 4; i++) {
        acc += g_7[i+3] * textureLoad(ping, vec2<i32>(i32(id.x), i32(id.y) + i), 0);
    }
    textureStore(pong, id.xy, acc);
}