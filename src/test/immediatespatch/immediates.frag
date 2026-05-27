#version 450

struct Inner {
    vec3 v;
    float arr[3];
};

layout(push_constant) uniform PushBlock {
    vec4 color;
    mat4 transform;
    float scale;
    Inner inner;
} pc;

layout(location = 0) in vec4 i_pos;
layout(location = 0) out vec4 o_color;

void main() {
    o_color = pc.color * pc.scale
            + pc.transform * i_pos
            + vec4(pc.inner.v, pc.inner.arr[1]);
}
