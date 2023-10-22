#version 450

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_uv;
layout (location = 2) in vec4 in_color;


layout (location = 0) out vec4 out_color;
layout (location = 1) out vec2 out_uv;
void main() {
    // Convert the co-ordinates from yakui coordinates to Vulkan:
    //
    // yakui: (0, 0) == top left, (1, 1) == bottom right
    // Vulkan: (-1, -1) === top left, (1, 1,) == bottom right
    gl_Position = vec4(in_pos * 2.0 - 1.0, 0.0, 1.0);
    out_color = in_color;
    out_uv = in_uv;
}
