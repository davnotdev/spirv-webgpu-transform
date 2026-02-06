#version 450

// This example is quite rare and actually requires the following extension.
// See [this issue](https://github.com/KhronosGroup/GLSL/issues/57).
#extension GL_EXT_shader_image_load_formatted : enable

layout(location = 0) flat in ivec3 i_coord;

layout(set = 0, binding = 0) uniform readonly imageCube u_icr;
layout(set = 0, binding = 1) uniform writeonly imageCube u_icw;

void indirect_load_write(readonly imageCube r, writeonly imageCube w) {
    vec4 s = imageLoad(r, i_coord);
    imageStore(w, i_coord, s);
}

void main() {
    indirect_load_write(u_icr, u_icw);
}

