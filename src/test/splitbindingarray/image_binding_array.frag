#version 440

#define MAX_IMAGES 8

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0, rgba8) uniform readonly image2D u_images[MAX_IMAGES];

void main() {
    o_color = vec4(0.0);
    for (int i = 0; i < MAX_IMAGES; i++) {
        o_color += imageLoad(u_images[i], ivec2(0, 0));
    }
}
