#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec4 color;


layout (location = 0) out vec4 o_color;
void main() {
    gl_Position = vec4(pos, 0., 1.0);
    o_color = color;
}