#version 420 core

in vec2 FragPos;

out vec4 FragColor;

uniform sampler2D fbuf;
uniform vec2 resolution;
uniform float aspect_ratio;
uniform vec2 fov;

mat3 rgb_to_xyz = mat3(
    vec3(0.4124564, 0.2126729, 0.0193339),
    vec3(0.3575761, 0.7151522, 0.1191920),
    vec3(0.1804375, 0.0721750, 0.9503041)
);

/*mat3 rgb_to_xyz = mat3(
    vec3(0.0193339, 0.2126729, 0.4124564),
    vec3(0.1191920, 0.7151522, 0.3575761),
    vec3(0.9503041, 0.0721750, 0.1804375)
);*/

mat3 xyz_to_rgb = mat3(
    vec3(3.2404542, -0.9692660, 0.0556434),
    vec3(-1.5371385, 1.8760108, -0.2040259),
    vec3(-0.4985314, 0.0415560, 1.0572252)
);

/*mat3 xyz_to_rgb = mat3(
    vec3(0.0556434, -0.9692660, 3.2404542),
    vec3(-0.2040259, 1.8760108, -1.5371385),
    vec3(1.0572252, 0.0415560, -0.4985314)
);*/

vec3 xyz_to_xyY(vec3 color) {
    float sum = color.x + color.y + color.z;
    return vec3(color.x / sum, color.y / sum, color.y);
}

vec3 xyY_to_xyz(vec3 color) {
    return vec3((color.z / color.y) * color.x, color.z, (color.z / color.y) * (1.0 - color.x - color.y));
}

void main() {

    vec3 numerator;
    float denominator;

    ivec2 px_coords = ivec2(FragPos * resolution);

    for (int x=-12; x<=12; x++) {
        for (int y=-12; y<=12; y++) {
            if (y==0 && x==0) {
                continue;
            }
            vec2 sample_offset = vec2(x, y) * (2.5/2160.0) / vec2(aspect_ratio, 1.0);
            float theta = length(sample_offset) * 70.0;//radians_per_px;
            float a = cos(theta) / (theta * theta);
            denominator += a;
            numerator += texture(fbuf, sample_offset + FragPos, 0).rgb * a;
        }
    }

    vec3 color = 0.913 * texelFetch(fbuf, px_coords, 0).rgb + 0.087 * (numerator / vec3(denominator));
    //FragColor = vec4(total, 1.0);
    vec3 xyz_color = rgb_to_xyz * color;
    float scotopic_luminance = xyz_color.y * (1.33 * (1.0 + (xyz_color.y + xyz_color.z) / xyz_color.x) - 1.68);
    //xyz_color.y = 0.000000000001;
    color = mix(vec3(scotopic_luminance), color, 0.8);
    //color = raw_color;
    //color *= 500000000.0;
    color = vec3(1.0) - exp(-color * 10000000000.0);
    color = pow(color, vec3(1.0 / 2.2));
    FragColor = vec4(color, 1.0);
}