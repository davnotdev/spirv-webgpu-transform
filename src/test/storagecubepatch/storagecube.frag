#version 450

layout(location = 0) flat in ivec3 i_coord;

layout(rgba32f, set = 0, binding = 0) uniform readonly imageCube u_icr;
layout(rgba32f, set = 0, binding = 1) uniform writeonly imageCube u_icw;

void main() {
    vec4 s = imageLoad(u_icr, i_coord);
    imageStore(u_icw, i_coord, s);
}

