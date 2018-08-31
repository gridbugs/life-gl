#version 150 core

in vec2 v_PixelCoord;
in vec2 v_Coord;
out vec4 Target;

uniform Properties {
    vec2 u_SizeInPixels;
    uint u_SurviveMin;
    uint u_SurviveMax;
    uint u_ResurrectMin;
    uint u_ResurrectMax;
};

uniform sampler2D t_InColour;

const float EPSILON = 0.001;

bool is_cell_alive(vec2 coord) {
    if (coord.x < -EPSILON || coord.y < -EPSILON ||
            coord.x >= u_SizeInPixels.x + EPSILON ||
            coord.y >= u_SizeInPixels.y + EPSILON) {
        return false;
    }
    vec4 colour = texture(t_InColour, coord / u_SizeInPixels);
    return colour.r > 0.5;
}

void main() {
    vec2 pixel_coord = v_PixelCoord;

    uint num_alive_neighbours =
        uint(is_cell_alive(pixel_coord + vec2(-1., -1.))) +
        uint(is_cell_alive(pixel_coord + vec2(0., -1.))) +
        uint(is_cell_alive(pixel_coord + vec2(1., -1.))) +
        uint(is_cell_alive(pixel_coord + vec2(1., 0.))) +
        uint(is_cell_alive(pixel_coord + vec2(1., 1.))) +
        uint(is_cell_alive(pixel_coord + vec2(0., 1.))) +
        uint(is_cell_alive(pixel_coord + vec2(-1., 1.))) +
        uint(is_cell_alive(pixel_coord + vec2(-1., 0.)));

    float alive;
    if (is_cell_alive(pixel_coord)) {
        if (num_alive_neighbours >= u_SurviveMin && num_alive_neighbours <= u_SurviveMax) {
            alive = 1.;
        } else {
            alive = 0.;
        }
    } else {
        if (num_alive_neighbours >= u_ResurrectMin && num_alive_neighbours <= u_ResurrectMax) {
            alive = 1.;
        } else {
            alive = 0.;
        }
    }
    Target = vec4(alive, alive, alive, 1.);
}
