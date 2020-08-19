#version 140

in vec3 position;

out vec3 reflect_dir;

uniform mat4 mvp;

void main() {
    reflect_dir = position;
    gl_Position = (mvp * vec4(position, 1.0)).xyww;
}
