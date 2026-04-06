#version 450

layout(set = 0, binding = 0) uniform sampler u_sampler;

layout(set = 0, binding = 1) uniform texture2D u_unused;
layout(set = 0, binding = 2) uniform texture2D u_used;

float inner(texture2D used) {
    // NOTE: Shader translation currently cannot handle these nested cases.
    // return textureProj(sampler2DShadow(used, u_sampler), vec4(0.0, 0.0, 0.0, 0.0));
    // This patch is designed to solve ^^^, but this is equivalent:
    return texture(sampler2D(used, u_sampler), vec2(0.0, 0.0)).x;

}

float outer(texture2D used) {
    return inner(used);
}

void main() {
    float g0 = inner(u_used);
}

