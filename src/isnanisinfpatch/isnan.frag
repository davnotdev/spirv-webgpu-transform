#version 450

bool _isnan(float x) {
    uint bits = floatBitsToUint(x);
    uint exp = (bits >> 23) & 0xffu;
    uint frac = bits & 0x7fffffu;
    return exp == 0xffu && frac == 0u;
}

void main() {
    float a = 0.0 / 0.0;
    bool b = _isnan(a);
}

