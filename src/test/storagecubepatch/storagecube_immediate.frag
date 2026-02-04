#version 450

layout(location = 0) flat in ivec3 i_coord;
layout(location = 1) flat in ivec3 i_coord2;

layout(rgba32f, set = 0, binding = 0) uniform readonly imageCube u_icr;
layout(rgba32f, set = 0, binding = 1) uniform writeonly imageCube u_icw;

void main() {
    vec4 s0 = imageLoad(u_icr, ivec3(i_coord.xy, 0));
    vec4 s1 = imageLoad(u_icr, i_coord + i_coord2) + s0;
    imageStore(u_icw, i_coord, s1);
}

