#version 450

bool ra, rb;
bvec2 rc, rd;

void main() {
    float a = 0.0;
    float b = 1.0;
    ra = isnan(a - b);
    rb = isinf(a - b);

    vec2 va = vec2(0.0, 0.0);
    vec2 vb = vec2(1.0, 0.0);
    rc = isnan(va - vb);
    rd = isinf(va - vb);
}

