#version 450

// Only difference here is that storage textures should not be effected.
layout(rgba32f, set = 0, binding = 4) uniform writeonly image2D u_no_effect;

layout(set = 0, binding = 0) uniform sampler u_unused_sampler;
layout(set = 0, binding = 1) uniform sampler u_sampler;

layout(set = 0, binding = 2) uniform texture2D u_unused_texture;
layout(set = 0, binding = 3) uniform texture2D u_used;

void main() {
    float g0 = textureProj(sampler2DShadow(u_used, u_sampler), vec4(0.0, 0.0, 0.0, 0.0));
    imageStore(u_no_effect, ivec2(0, 0), vec4(g0, 0.0, 0.0, 0.0));
}

