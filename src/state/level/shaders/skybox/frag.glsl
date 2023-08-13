#version 140

in vec3 reflect_dir;

out vec4 color;

uniform samplerCube cubetex;

void main() {
    color = texture(cubetex, reflect_dir);
}
