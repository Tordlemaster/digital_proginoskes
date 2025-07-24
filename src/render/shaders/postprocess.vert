#version 420 core
layout (location = 0) in vec2 aPos;

out vec2 FragPos;

void main() {
    FragPos = aPos / 2.0 + 0.5;
    gl_Position = vec4(aPos, 0.0, 1.0);
}