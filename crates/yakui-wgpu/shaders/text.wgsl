struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var coverage_texture: texture_2d<f32>;
@group(0) @binding(1) var coverage_sampler: sampler;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    var adjusted: vec2<f32> = in.position;
    adjusted *= vec2(2.0, -2.0);
    adjusted += vec2(-1.0, 1.0);

    out.position = vec4<f32>(adjusted, 0.0, 1.0);
    out.texcoord = in.texcoord;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = textureSample(coverage_texture, coverage_sampler, in.texcoord);

    let alpha = max(max(coverage.r, coverage.g), coverage.b) * in.color.a * coverage.a;
    let has_color = step(0.05, max(max(in.color.r, in.color.g), max(in.color.b, in.color.a)));

    let color = in.color.rgb * has_color * alpha + coverage.rgb * (1.0 - has_color);

    return vec4(color, alpha);
}