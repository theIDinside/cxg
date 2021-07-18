#version 430 core
layout (location = 0) in vec2 vertex;
layout (location = 1) in vec4 color;

uniform mat4 projection;

out vec4 rect_color;

void main()
{
    gl_Position = projection * vec4(vertex, 0.0, 1.0);
    rect_color = vec4(color);
}