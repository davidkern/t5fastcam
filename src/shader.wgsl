struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    let u = x / 2.0 + 1.0;
    let v = y / 2.0 + 1.0;

    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texture_uv = vec2<f32>(u, v);

    return out;
}

@group(0) @binding(0)
var t_video: texture_2d<f32>;
@group(0) @binding(1)
var s_video: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_video, s_video, in.texture_uv);
}
