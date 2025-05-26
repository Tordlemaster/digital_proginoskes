#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

in vec3 transformation;

out vec2 TexCoord;

void main()
{
    TexCoord = aTexCoord;

    vec2 coords = transformation * aPos;
    gl_Position = vec4(coords.x, coords.y, 0.0, 1.0);
}