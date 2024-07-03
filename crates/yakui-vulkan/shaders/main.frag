#version 450
#define NO_TEXTURE 4294967295
#define WORKFLOW_MAIN 0
#define WORKFLOW_TEXT 1

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_uv;
layout (location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D textures[1000];
layout(push_constant) uniform push_constants {
    uint texture_id;
    uint workflow;
};

void main() {
    if (texture_id == NO_TEXTURE) {
        out_color = in_color;
        return;
    } 
    
    if (workflow == WORKFLOW_TEXT) {
        vec4 coverage = texture(textures[texture_id], in_uv);

        float alpha = max(max(coverage.r, coverage.g), coverage.b) * in_color.a * coverage.a;
        float has_color = step(0.05, max(max(in_color.r, in_color.g), max(in_color.b, in_color.a)));
        vec3 color = in_color.rgb * has_color * alpha + coverage.rgb * (1.0 - has_color);

        out_color = vec4(color, alpha);
    } else {
        vec4 user_texture = texture(textures[texture_id], in_uv);
        out_color = in_color * user_texture;
    }
}
