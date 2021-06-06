#version 430 core
layout (location = 0) in vec2 vertex;
uniform mat4 projection;

void main()
{
    gl_Position = projection * vec4(vertex, 0.0, 1.0);
}