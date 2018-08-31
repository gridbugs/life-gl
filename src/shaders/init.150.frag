#version 150 core

in vec2 v_PixelCoord;
out vec4 Target;

uniform Properties {
    float u_Seed;
    vec2 u_SizeInPixels;
};

const float PHI = 1.61803398874989484820459 * 00000.1;
const float PI  = 3.14159265358979323846264 * 00000.1;
const float SQ2 = 1.41421356237309504880169 * 10000.0;

float gold_noise(vec2 coord, float seed) {
    return fract(tan(distance(coord*(seed+PHI), vec2(PHI, PI)))*SQ2);
}

void main() {
    float alive;
    if (gold_noise(v_PixelCoord, u_Seed) > 0.5) {
        alive = 0;
    } else {
        alive = 1;
    }
    Target = vec4(alive, alive, alive, 1);
}
