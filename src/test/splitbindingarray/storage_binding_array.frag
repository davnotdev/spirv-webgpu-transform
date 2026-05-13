#version 440

#define MAX_THINGS 8

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0, std140) buffer Thing {
      vec4 thing1;
      float thing2;
} u_things[MAX_THINGS];

void main() {
    o_color = vec4(0.0);
    for (int i = 0; i < MAX_THINGS; i++) {
        o_color += u_things[i].thing1;
        o_color.x += u_things[i].thing2;
    }
}

