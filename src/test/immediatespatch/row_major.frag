#version 450

layout(push_constant) uniform PC {
    layout(row_major) mat4 m;
    vec4 offset;
} pc;
layout(location = 0) in vec4 input_vec;
layout(location = 1) out vec4 o_color;

void main() {
    o_color = pc.m * input_vec + pc.offset;
}
