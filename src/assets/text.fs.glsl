#version 430 core
in vec2 TexCoords;
in vec3 TCol;

out vec4 color;

uniform sampler2D text;

void main()
{    
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text, TexCoords).r);
    color = vec4(TCol, 1.0) * sampled;
}