#version 140

in vec2 o_tex_coords;
in vec3 o_normal;

uniform sampler2D tex;

const vec3 LIGHT = vec3(0.2, -0.8, 0.1);

void main() {
    float lum = max(dot(normalize(o_normal), normalize(LIGHT)), 0.0);
    vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
    vec4 tex_color = texture2D(tex, o_tex_coords);
    gl_FragColor = vec4(tex_color.xyz * color, tex_color.w);
}
