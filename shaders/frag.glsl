#version 140
in vec2 o_tex_coords;
uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, o_tex_coords);
}
