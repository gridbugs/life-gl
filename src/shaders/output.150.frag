#version 150 core

out vec4 Target;
in vec2 v_Coord;

uniform sampler2D t_InColour;

uniform Properties {
    vec4 u_AliveColour;
    vec4 u_DeadColour;
};


vec4 get_colour() {
    if (texture(t_InColour, v_Coord).r > 0.5) {
        return u_AliveColour;
    } else {
        return u_DeadColour;
    }
}

void main() {
    Target = get_colour();
}
