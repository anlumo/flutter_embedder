struct FlutterRenderUniform {
    offset: vec2<f32>,
    size: vec2<f32>,
}

@group(0) @binding(0)
var t_flutter: texture_2d<f32>;
@group(0) @binding(1)
var s_flutter: sampler;

@group(1) @binding(0)
var<uniform> render_information: FlutterRenderUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_flutter, s_flutter, (in.vert_pos + 1.0) * 0.5);
}
