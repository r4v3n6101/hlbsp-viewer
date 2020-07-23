#version 140

in vec3 position;
in vec2 tex_coords;

out vec2 o_tex_coords;

uniform mat4 proj;
uniform mat4 view;
uniform vec3 origin;

void main() {
    o_tex_coords =  tex_coords;
    vec3 pos = (position - origin) * 0.001;
    gl_Position = proj * view * vec4(vec3(pos.x, pos.z, -pos.y), 1.0);
}

