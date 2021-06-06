#version 430 core
out vec4 FragColor;
uniform vec4 fillcolor;

void main()
{
    FragColor = vec4(fillcolor);
}