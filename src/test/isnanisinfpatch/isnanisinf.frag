#version 450

bool ra;

void main() {
    float t0 = 0.0f/0.0f;
    ra = isnan(t0) || isinf(t0);
}

