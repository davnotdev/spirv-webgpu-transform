#version 440

#define MAX_TEXTURES 8

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform texture2DArray u_textures[MAX_TEXTURES];
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    o_color = vec4(0.0);
    for (int i = 0; i < MAX_TEXTURES; i++) {
        o_color += texture(sampler2DArray(u_textures[i], u_sampler), vec3(0.0, 0.0, 0.0));
    }
}
