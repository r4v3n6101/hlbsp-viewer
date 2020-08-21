#version 140

in vec2 o_tex_coords;
in vec3 o_normal;

uniform sampler2D tex;

void main() {
    vec2 tex_size = textureSize(tex, 0);
    gl_FragColor = texture(tex, o_tex_coords / tex_size);
}
