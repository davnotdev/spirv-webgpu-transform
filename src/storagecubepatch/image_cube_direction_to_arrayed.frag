#version 450

layout(set = 0, binding = 0) uniform writeonly imageCube u_ic;
layout(set = 0, binding = 1) uniform writeonly image2DArray u_i2a;

// See [Section 3.9.10, GL 4.2](https://registry.khronos.org/OpenGL/specs/gl/glspec42.core.pdf)
ivec3 _imageCubeDirectionToArrayed(ivec3 r)
{
    ivec3 a = abs(r);
    bool x = a.x >= a.y && a.x >= a.z;
    bool y = a.y >  a.x && a.y >= a.z;

    int face =
          x ? (r.x > 0 ? 0 : 1)
        : y ? (r.y > 0 ? 2 : 3)
            : (r.z > 0 ? 4 : 5);

    ivec2 st =
          face == 0 ? ivec2(-r.z, -r.y) :
          face == 1 ? ivec2( r.z, -r.y) :
          face == 2 ? ivec2( r.x,  r.z) :
          face == 3 ? ivec2( r.x, -r.z) :
          face == 4 ? ivec2( r.x, -r.y) :
                      ivec2(-r.x, -r.y);

    return ivec3(st, face);
}


void main() {
    ivec3 coord = ivec3(1, -2, 3);
    vec4 value = vec4(0.0, 0.0, 0.0, 0.0);
    imageStore(u_ic, coord, value);
    imageStore(u_i2a, _imageCubeDirectionToArrayed(coord), value);
}

