#version 450

layout(set = 0, binding = 0) uniform sampler u_sampler;

layout(set = 0, binding = 1) uniform texture2D u_unused;
layout(set = 0, binding = 2) uniform texture2D u_used;

float inner(texture2D used) {
    return textureProj(sampler2DShadow(used, u_sampler), vec4(0.0, 0.0, 0.0, 0.0));
}

float outer(texture2D used) {
    return inner(used);
}

void main() {
    float g0 = inner(u_used);
}

