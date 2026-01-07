#version 450

bool ra;

void main() {
    // Originally, this had a 0.0f/0.0f, but WGSL can't represent NAN.
    float t0 = 0.0f/10.0f;
    ra = isnan(t0) || isinf(t0);
}

