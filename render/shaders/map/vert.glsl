#version 140

in vec3 position;
in vec2 tex_coords;
in vec3 normal;

out vec2 o_tex_coords;
out vec3 o_normal;

uniform mat4 mvp;

const float SCALE_FACTOR = 0.0007;

void main() {
    o_tex_coords = tex_coords;
    o_normal = normal;

    vec3 pos = position * SCALE_FACTOR; // TODO : replace with model matrix
    gl_Position = mvp * vec4(vec3(pos.x, pos.z, -pos.y), 1.0);
}

