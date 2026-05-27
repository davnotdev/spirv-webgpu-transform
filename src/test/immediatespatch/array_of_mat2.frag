#version 450

layout(push_constant) uniform PC {
    mat2 mats[2];
} pc;
layout(location = 0) out vec4 o_color;

void main() {
    o_color = vec4(pc.mats[0][0] + pc.mats[1][0], 0.0, 1.0);
}
