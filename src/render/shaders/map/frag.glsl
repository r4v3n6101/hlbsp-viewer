#version 140

in vec2 o_tex_coords;
in vec2 o_light_tex_coords;

flat in uint o_lightmap_offset;
flat in uvec2 o_lightmap_size;

uniform sampler2D colormap;
uniform samplerBuffer lightmap;

const bool BILINEAR = true;

vec4 sample_lightmap(in vec2 uv) {
    int offset = int(o_lightmap_offset + floor(uv.y) * o_lightmap_size.x + floor(uv.x));
    return texelFetch(lightmap, offset);
}

vec4 sample_bilinear_lightmap(in vec2 uv) {
    vec2 uv2 = min(uv + 1, o_lightmap_size);

    vec4 tl = sample_lightmap(uv);
    vec4 tr = sample_lightmap(vec2(uv2.x, uv.y));
    vec4 bl = sample_lightmap(vec2(uv.x, uv2.y));
    vec4 br = sample_lightmap(uv2); 

    vec2 f = fract(uv);
    vec4 tA = mix(tl, tr, f.x);
    vec4 tB = mix(bl, br, f.x);

    return mix(tA, tB, f.y);
}

void main() {
    vec4 color = texture(colormap, o_tex_coords / textureSize(colormap, 0));
    if (BILINEAR) {
        color *= sample_bilinear_lightmap(o_light_tex_coords);
    } else {
        color *= sample_lightmap(o_light_tex_coords);
    }
    gl_FragColor = color;
}
