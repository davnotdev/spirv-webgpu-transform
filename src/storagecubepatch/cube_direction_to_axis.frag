#version 450

// See [Section 3.9.10, GL 4.2](https://registry.khronos.org/OpenGL/specs/gl/glspec42.core.pdf)
int _cubemapDirectionToAxis(vec3 r) {
    vec3 a = abs(r);
    if (a.x >= a.y && a.x >= a.z) {
        return r.x > 0 ? 0 : 1;
    } else if (a.y >= a.x && a.y >= a.z) {
        return r.y > 0 ? 2 : 3;
    } else {
        return r.z > 0 ? 4 : 5;
    }
}


void main() {
    int _a = _cubemapDirectionToAxis(vec3(1.0, -2.0, 3.0));
}

