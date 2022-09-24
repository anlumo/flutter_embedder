struct FlutterRenderUniform {
    offset: vec2<f32>,
    size: vec2<f32>,
    viewport: vec2<f32>,
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
    out.vert_pos = vec2<f32>(
        f32(in_vertex_index & 1u),
        f32((in_vertex_index >> 1u) & 1u),
    ) * 2.0 - 1.0;
    // 0 (-1, -1)
    // 1 (-1,  1)
    // 2 ( 1, -1)
    // 3 ( 1,  1)

    out.clip_position = vec4<f32>(
        out.vert_pos * render_information.size / render_information.viewport + render_information.offset / render_information.viewport,
        0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_flutter, s_flutter, (in.vert_pos * vec2(1.0, -1.0) + 1.0) * 0.5);
}
