#version 430 core
layout (location = 0) in vec4 vertex; // <vec2 pos, vec2 uv>
layout (location = 1) in vec4 color;  // <vec3 col, float interpolate>

uniform mat4 projection;
uniform vec2 rect_size;
uniform bool use_texture;

// out vec4 rect_color;    // RGB color 
// out vec2 u_size;        // size of window
// out vec2 texture_coordinates;
// out vec2 texel_coords;  // texel (fragment) which we reference in the fragment shader, to calculate SDF, inside box

out RectangleInfo {
    vec2 texel_coordinates;
    vec2 tex_coords;
    vec2 size;
    vec4 color;
    bool use_texture;
} rectangleInfo;

void main()
{    
    gl_Position = projection * vec4(vertex.xy, 0.0, 1.0);
    // rect_color = color;
    // texture_coordinates = vertex.zw;
    // texel_coords = vertex.xy;
    // u_size = rect_size;

    rectangleInfo.texel_coordinates = vertex.xy;
    rectangleInfo.tex_coords = vertex.zw;
    rectangleInfo.color = color;
    rectangleInfo.size = rect_size;
    rectangleInfo.use_texture = use_texture;

}