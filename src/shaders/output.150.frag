#version 150 core

out vec4 Target;
in vec4 v_Colour;

uniform sampler2D t_InColour0;
uniform sampler2D t_InColour1;

uniform Properties {
    uint u_WhichInputToRenderFrom;
};

in vec2 v_Coord;

vec4 get_colour(sampler2D t_InColour) {
    if (texture(t_InColour, v_Coord).r > 0) {
        return vec4(0, 0, 0, 1);
    } else {
        return vec4(1, 1, 1, 1);
    }
}

void main() {
    if (u_WhichInputToRenderFrom == 0u) {
        Target = get_colour(t_InColour0);
    } else {
        Target = get_colour(t_InColour1);
    }
}
