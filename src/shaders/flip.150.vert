#version 150 core

in vec2 a_CornerZeroToOne;
out vec2 v_Coord;

void main() {
    v_Coord = a_CornerZeroToOne;
    float x = a_CornerZeroToOne.x * 2 - 1;
    float y = 1 - a_CornerZeroToOne.y * 2;
    gl_Position = vec4(x, y, 0, 1);
}
