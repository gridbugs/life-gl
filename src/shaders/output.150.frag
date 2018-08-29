#version 150 core

out vec4 Target;
in vec4 v_Colour;

uniform sampler2D t_InColour0;
uniform sampler2D t_InColour1;

uniform Properties {
    uint u_WhichInputToRenderFrom;
};

in vec2 v_Coord;

void main() {
    if (u_WhichInputToRenderFrom == 0u) {
        Target = texture(t_InColour0, v_Coord);
    } else {
        Target = texture(t_InColour1, v_Coord);
    }
}
