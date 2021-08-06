#version 430 core
layout (location = 0) in vec4 vertex; // <vec2 pos, vec2 uv>
layout (location = 1) in vec4 color;  // color2

// todo: move this into a uniform buffer object, because this uniform
// will be the same across all shader programs
uniform mat4 projection;

uniform vec2 rect_size;
uniform float useTexture;

out RectangleInfo {
    vec2 tex_coords;
    vec2 size;
    vec4 color;
    float use_texture;
} rectangleInfo;

void main()
{    
    gl_Position = projection * vec4(vertex.xy, 0.0, 1.0);
    // *really* clean and nice shader now
    rectangleInfo.tex_coords = vertex.zw;
    rectangleInfo.color = color;
    rectangleInfo.size = rect_size;
    rectangleInfo.use_texture = useTexture;

}