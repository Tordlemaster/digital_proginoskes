#version 420 core

in vec2 FragPos;

out vec4 FragColor;

uniform sampler2D fbuf;

vec2 resolution = vec2(3840, 2160);

void main() {
    FragColor = texelFetch(fbuf, ivec2(FragPos * resolution), 0); //vec4(1.0, 0.0, 0.0, 1.0);
}