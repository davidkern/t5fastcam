struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_uv: vec2<f32>,
}

struct SequenceUniform {
    sequence: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 4>(
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0,  1.0)
    );

    let x = pos[in_vertex_index][0];
    let y = pos[in_vertex_index][1];
    let u = (x + 1.0) / 2.0;
    let v = (y + 1.0) / 2.0;

    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texture_uv = vec2<f32>(u, v);

    return out;
}

@group(0) @binding(0)
var t_video: texture_2d<f32>;
@group(0) @binding(1)
var s_video: sampler;
@group(0) @binding(2)
var<uniform> u_sequence: SequenceUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color_wheel = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );
    let phase = u32(u_sequence.sequence) % 3u;
    let color = color_wheel[phase];

    let lum = textureSample(t_video, s_video, in.texture_uv)[0];
    return vec4<f32>(lum*color[0], lum*color[1], lum*color[2], 1.0);
}
