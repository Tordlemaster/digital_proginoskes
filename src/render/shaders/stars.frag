#version 420 core

out vec4 FragColor;

in vec3 Pos;
in vec3 Color;



void main() {
    FragColor = vec4(vec3((1.0 - distance(gl_PointCoord, vec2(0.5)))), 1.0);
}