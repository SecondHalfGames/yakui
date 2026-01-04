struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;

    let uv = vec2(
        f32((idx << 1u) & 2u),
        f32(idx & 2u)
    );

    out.pos = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
    out.texcoord = vec2(uv.x, 1.0 - uv.y);
    return out;
} 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(source_texture, source_sampler, in.texcoord);

    return vec4(color.rgb * color.a, color.a);
}