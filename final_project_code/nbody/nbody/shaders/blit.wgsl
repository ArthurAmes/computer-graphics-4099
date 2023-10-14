@group(0) @binding(0)
var<uniform> res: vec2f;
@group(0) @binding(1)
var<uniform> res_internal: vec2f;
@group(0) @binding(2)
var final_image: texture_2d<f32>;
@group(0) @binding(3)
var final_sampler: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let texcoords = pos.xy/res_internal.xy;
    return textureSample(final_image, final_sampler, texcoords);
}