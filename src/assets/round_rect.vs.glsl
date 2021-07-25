#version 430 core
layout (location = 0) in vec2 vertex;
layout (location = 1) in vec4 color;

uniform mat4 projection;
uniform vec2 rect_size;
out vec4 rect_color;
out vec2 u_size;
out vec2 texel_coords;

void main()
{    
    gl_Position = projection * vec4(vertex, 0.0, 1.0);
    rect_color = color;
    texel_coords = vertex;
    u_size = rect_size;
}