#version 150 core

out vec4 Target;
in vec4 v_Colour;

uniform Properties {
    uint u_Init;
    float u_Delta;
};

uniform sampler2D t_InColour;

in vec2 v_Coord;

void main() {
    if (u_Init == 1u) {
        Target = vec4(1, 0, 0, 1);
    } else {
        Target = texture(t_InColour, v_Coord) + vec4(u_Delta, 0, 0, 0);
    }
}
