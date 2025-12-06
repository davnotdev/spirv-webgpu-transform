#version 450

// In the real world, omitted bindings are likely.

layout(set = 0, binding = 0) uniform sampler u_regular_sampler;
// layout(set = 0, binding = 1) uniform sampler u_comparison_sampler;

layout(set = 0, binding = 2) uniform texture2D u_mixed_texture;

// layout(set = 0, binding = 3) uniform texture2D u_other_a;
layout(set = 0, binding = 4) uniform texture2D u_other_b;

void main() {
    // There will be no splitting here.
    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
}

