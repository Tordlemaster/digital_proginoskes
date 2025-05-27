#version 330 core

in vec2 TexCoord;

out vec4 FragColor;

uniform float star_count;
uniform int layer;

void main()
{
    FragColor = vec4(float(layer==0), float(layer==1), float(layer==2), 1.0f);
    //FragColor = vec4(star_count, 0.0, 0.0, 1.0);
}