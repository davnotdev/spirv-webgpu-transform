#version 440

#define MAX_TEXTURES 8

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform texture2D u_textures[MAX_TEXTURES];
layout(set = 0, binding = 1) uniform sampler u_samplers[MAX_TEXTURES];

void main() {
    o_color = vec4(0.0);
    for (int i = 0; i < MAX_TEXTURES; i++) {
        // We want to ensure that this is valid even if opaque values never resolve. 
        texture(sampler2D(u_textures[i], u_samplers[i]), vec2(0.0, 0.0));
    }
}

