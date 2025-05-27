#version 330 core

in vec2 TexCoord;

out vec4 FragColor;

uniform float star_count;
uniform int layer;

vec3[] colors = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0),
    vec3(1.0, 0.0, 1.0)
);

void main()
{
    //FragColor = vec4(vec3(float(layer==0), float(layer==1), float(layer==2)), 1.0f);
    //FragColor = vec4(colors[min(layer, 5)] * min(min(TexCoord.x - 0.0, 1.0 - TexCoord.x), min(TexCoord.y - 0.0, 1.0 - TexCoord.y)), 1.0);
    FragColor = vec4(star_count, 0.0, 0.0, 1.0);
}