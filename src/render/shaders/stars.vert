#version 420 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

layout (std140, binding = 0) uniform Cam {
    mat4 proj_view;
};

uniform mat4 cam_proj_view;

out vec3 Pos;
out vec3 Color;

void main() {
    Pos = aPos;
    Color = aColor;
    gl_Position = cam_proj_view * vec4(aPos, 1.0);
}