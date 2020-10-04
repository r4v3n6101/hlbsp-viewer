#version 140

in vec3 position;
in vec2 tex_coords;
in vec2 light_tex_coords;

in uint lightmap_offset;
in uvec2 lightmap_size;

in vec3 normal;

out vec2 o_tex_coords;
out vec2 o_light_tex_coords;

flat out uint o_lightmap_offset;
flat out uvec2 o_lightmap_size;

uniform mat4 mvp;
uniform vec3 origin;

const mat4 ROTATE_MAT = mat4(
	0, 0, -1, 0, 
	-1, 0, 0, 0, 
	0, 1, 0, 0, 
	0, 0, 0, 1
);

void main() {
    o_tex_coords = tex_coords;
    o_light_tex_coords = light_tex_coords;
    o_lightmap_offset = lightmap_offset;
    o_lightmap_size = lightmap_size;

    gl_Position = mvp * ROTATE_MAT * vec4(origin + position, 1.0);
}
