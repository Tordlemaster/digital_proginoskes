#version 420 core
layout (location = 0) in vec3 aPos;

layout (std140, binding = 0) uniform Cam {
    mat4 proj_view;
};

out vec3 Pos;

void main() {
    Pos = aPos;
    gl_Position = proj_view * vec4(aPos, 1.0);
}