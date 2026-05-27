#version 450
layout(push_constant) uniform PC {
    mat2 m;
    float x;
} pc;
layout(location = 0) out vec4 o_color;

void main() {
    vec2 col0 = pc.m[0];
    vec2 col1 = pc.m[1];
    o_color = vec4(col0, col1) * pc.x;
}
