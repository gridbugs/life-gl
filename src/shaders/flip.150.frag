#version 150 core

in vec2 v_Coord;
uniform sampler2D t_InColour;
out vec4 Target;

void main() {
    Target = texture(t_InColour, v_Coord);
}
