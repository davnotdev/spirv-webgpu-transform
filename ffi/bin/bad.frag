#version 450

layout(location = 0) out vec4 o_color;
layout(location = 0) in vec2 i_tex_coord;

layout(set = 0, binding = 0) uniform sampler2D u_comb1;
layout(set = 0, binding = 1) uniform sampler2D u_comb2;
layout(set = 1, binding = 0) uniform sampler2D u_comb3;
layout(set = 2, binding = 0) uniform writeonly imageCube u_ic;

void main() {
    vec4 res = texture(u_comb1, i_tex_coord) + texture(u_comb2, i_tex_coord) + texture(u_comb3, i_tex_coord);
    imageStore(u_ic, ivec3(1, 2, 3), res);
    o_color = res;
}
