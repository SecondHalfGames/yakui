#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 in_color;
layout (location = 1) in vec2 in_uv;
layout (location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D textures[1000];

void main() {
    out_color = texture(textures[0], in_uv) * in_color;
}