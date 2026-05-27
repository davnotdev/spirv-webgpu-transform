#version 450

struct Level2 {
    float a;
    vec3 b;
};
struct Level1 {
    Level2 deep;
    vec2 c;
};
layout(push_constant) uniform PC {
    Level1 outer;
    float scale;
} pc;
layout(location = 0) out vec4 o_color;

void main() {
    vec3 v = pc.outer.deep.b * pc.outer.deep.a;
    o_color = vec4(v.x + pc.outer.c.x, v.y + pc.outer.c.y, v.z + pc.scale, 1.0);
}
