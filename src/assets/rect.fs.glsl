#version 430 core
out vec4 FragColor;
in vec4 rect_color;
uniform vec4 fillcolor;

void main()
{
    FragColor = rect_color;
}