#version 450

layout(set = 0, binding = 0) uniform sampler u_regular_sampler;
layout(set = 0, binding = 1) uniform sampler u_comparison_sampler;
layout(set = 0, binding = 2) uniform texture2D u_shadow_atlas;

float pcf(texture2D shadow, sampler comparison_sampler) {
    return textureProj(sampler2DShadow(shadow, comparison_sampler), vec4(0.0, 0.0, 0.0, 1.0));
}

void main() {
    vec4 g0 = textureLod(sampler2D(u_shadow_atlas, u_regular_sampler), vec2(0.0, 0.0), 0.0);
    float g1 = pcf(u_shadow_atlas, u_comparison_sampler);
}
