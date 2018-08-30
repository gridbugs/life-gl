#version 150 core

in vec2 a_CornerZeroToOne;
out vec4 v_Colour;
out vec2 v_Coord;
out vec2 v_PixelCoord;

uniform Properties {
    uint u_Init;
    float u_Delta;
    vec2 u_SizeInPixels;
};


void main() {

    float x = a_CornerZeroToOne.x * 2 - 1;
    float y = 1 - a_CornerZeroToOne.y * 2;

    v_Colour = vec4(1 - a_CornerZeroToOne.y, 0, a_CornerZeroToOne.y, 1);
    v_Coord = a_CornerZeroToOne;
    v_PixelCoord = a_CornerZeroToOne * (u_SizeInPixels - vec2(0, 0));
    gl_Position = vec4(x, y, 0, 1);
}
