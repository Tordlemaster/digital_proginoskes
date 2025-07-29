#version 420 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

layout (std140, binding = 0) uniform Cam {
    mat4 proj_view;
};

out vec3 Pos;
out vec2 screen_pos;
out vec3 raw_color;

void main() {
    Pos = aPos;
    raw_color = aColor;
    vec4 g = proj_view * vec4(aPos, 1.0);
    screen_pos = g.xy;
    gl_Position = g;
}