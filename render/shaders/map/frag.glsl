#version 140

in vec2 o_tex_coords;
in vec2 o_light_tex_coords;

uniform sampler2D colormap;
uniform sampler2D lightmap;

void main() {
    vec4 light = texture(lightmap, o_light_tex_coords / textureSize(lightmap, 0));
    gl_FragColor = texture(colormap, o_tex_coords / textureSize(colormap, 0));
}
