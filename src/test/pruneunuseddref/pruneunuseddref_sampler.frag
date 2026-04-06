#version 450

layout(set = 0, binding = 0) uniform texture2D u_texture;

layout(set = 0, binding = 1) uniform sampler u_unused;
layout(set = 0, binding = 2) uniform sampler u_used;

void main() {
    float g0 = textureProj(sampler2DShadow(u_texture, u_used), vec4(0.0, 0.0, 0.0, 0.0));
}


