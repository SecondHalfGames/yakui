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
    let coverage = textureSample(coverage_texture, coverage_sampler, in.texcoord).r;
    let alpha = coverage * in.color.a;

    return vec4(in.color.rgb * alpha, alpha);
}