#version 150 core

in vec2 a_CornerZeroToOne;
out vec2 v_PixelCoord;
out vec2 v_Coord;

uniform Properties {
    vec2 u_SizeInPixels;
    uint u_SurviveMin;
    uint u_SurviveMax;
    uint u_ResurrectMin;
    uint u_ResurrectMax;
};

void main() {
    v_Coord = a_CornerZeroToOne;
    float x = a_CornerZeroToOne.x * 2 - 1;
    float y = 1 - a_CornerZeroToOne.y * 2;
    v_PixelCoord = a_CornerZeroToOne * u_SizeInPixels;
    gl_Position = vec4(x, y, 0, 1);
}
