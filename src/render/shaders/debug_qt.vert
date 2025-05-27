#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

uniform mat3 transformation;
uniform int layer;

out vec2 TexCoord;

void main()
{
    TexCoord = aTexCoord;

    vec3 coords = transformation * vec3(aPos, 1.0);
    gl_Position = vec4(coords, 1.0);
    //gl_Position = vec4(coords.xy, float(layer), 1.0);
}