#version 140

in vec2 o_tex_coords;
in vec2 o_light_tex_coords;

flat in uint o_lightmap_offset;
flat in uvec2 o_lightmap_size;

uniform sampler2D colormap;
uniform samplerBuffer lightmap;

vec4 sample_lightmap() {
    int offset = int(o_lightmap_offset + floor(o_light_tex_coords.y) * o_lightmap_size.x + floor(o_light_tex_coords.x));
    return texelFetch(lightmap, offset);
}

void main() {
    vec2 texture_size = textureSize(colormap, 0);
    gl_FragColor = sample_lightmap() * texture(colormap, o_tex_coords / texture_size);
}
