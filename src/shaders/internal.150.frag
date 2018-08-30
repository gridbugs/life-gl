#version 150 core

out vec4 Target;
in vec4 v_Colour;

uniform Properties {
    uint u_Init;
    float u_Delta;
    vec2 u_SizeInPixels;
};

uniform sampler2D t_InColour;

in vec2 v_Coord;
in vec2 v_PixelCoord;

const uint SURVIVE_MIN = 2u;
const uint SURVIVE_MAX = 3u;
const uint RESURRECT_MIN = 3u;
const uint RESURRECT_MAX = 3u;

float rand(vec2 co) {
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

uint is_cell_alive(ivec2 coord) {
    if (coord.x < 0 || coord.y < 0 ||
            coord.x >= int(u_SizeInPixels.x) ||
            coord.y >= int(u_SizeInPixels.y)) {
        return 0u;
    }
    vec4 colour = texelFetch(t_InColour, coord, 0);
    if (colour.r > 0.5) {
        return 1u;
    } else {
        return 0u;
    }
}

void main() {
    if (u_Init == 1u) {
        float alive;
        if (rand(v_Coord) > 0.5) {
            alive = 0;
        } else {
            alive = 1;
        }
        Target = vec4(alive, alive, alive, 1);
    } else {
        if (u_Delta > 0.5) {
            vec2 coord = v_Coord;
            vec2 flipped_coord = vec2(coord.x, 1 - coord.y);
            Target = texture(t_InColour, flipped_coord);
            return;
        }
        vec2 flipped_pixel_coord = vec2(v_PixelCoord.x, u_SizeInPixels.y - v_PixelCoord.y);
        ivec2 pixel_coord = ivec2(flipped_pixel_coord);
        uint num_alive_neighbours =
            is_cell_alive(pixel_coord + ivec2(-1, -1)) +
            is_cell_alive(pixel_coord + ivec2(0, -1)) +
            is_cell_alive(pixel_coord + ivec2(1, -1)) +
            is_cell_alive(pixel_coord + ivec2(1, 0)) +
            is_cell_alive(pixel_coord + ivec2(1, 1)) +
            is_cell_alive(pixel_coord + ivec2(0, 1)) +
            is_cell_alive(pixel_coord + ivec2(-1, 1)) +
            is_cell_alive(pixel_coord + ivec2(-1, 0));

        bool cell_is_alive = is_cell_alive(pixel_coord) == 1u;

        float alive;
        if (cell_is_alive) {
            if (num_alive_neighbours >= SURVIVE_MIN && num_alive_neighbours <= SURVIVE_MAX) {
                alive = 1;
            } else {
                alive = 0;
            }
        } else {
            if (num_alive_neighbours >= RESURRECT_MIN && num_alive_neighbours <= RESURRECT_MAX) {
                alive = 1;
            } else {
                alive = 0;
            }
        }
        Target = vec4(alive, alive, alive, 1);
    }
}
