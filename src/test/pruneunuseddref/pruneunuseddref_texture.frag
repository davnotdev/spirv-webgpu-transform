#version 450

layout(set = 0, binding = 0) uniform sampler u_sampler;

layout(set = 0, binding = 1) uniform texture2D u_unused;
layout(set = 0, binding = 2) uniform texture2D u_used;

void main() {
    float g0 = textureProj(sampler2DShadow(u_used, u_sampler), vec4(0.0, 0.0, 0.0, 0.0));
}

