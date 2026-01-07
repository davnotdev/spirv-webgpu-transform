#version 450

bvec3 ra, rb;

void main() {
    vec3 t0 = vec3(5.0f, 7.7f, 3.4f);
    ra = isnan(t0);
    rb = isinf(t0);
}

