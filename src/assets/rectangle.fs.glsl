#version 430 core
out vec4 FragColor;
// The RGB Color
// in vec4 rect_color;
// The interpolation value; decides if we use the bound texture
// in float interpolation;
/// size of the rectangle currently getting drawn
// in vec2 u_size;
// from http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
float box_signed_distance_field_rounding(vec2 CenterPosition, vec2 Size, float Radius) {
    return length(max(abs(CenterPosition)-Size+Radius,0.0))-Radius;
}

uniform float radius;
uniform vec2 rect_pos;
uniform sampler2D texture_sampler;

in RectangleInfo {
    vec2 texel_coordinates;
    vec2 tex_coords;
    vec2 size;
    vec4 color;
    bool use_texture;
} rectangleInfo;

const float smoothness = 0.7;

void main()
{
    vec4 chosen_color = vec4(0);

    if(!rectangleInfo.use_texture) {
        chosen_color = rectangleInfo.color;
    } else {
        vec4 sampledTexture = texture(texture_sampler, rectangleInfo.tex_coords);
        chosen_color = mix(sampledTexture, rectangleInfo.color, 0.005);
    }
    
    if(radius > 0.0) {
        // The pixel space scale of the rectangle.       
        // the pixel space location of the rectangle.
        vec2 location = rect_pos;
        float boundary = rectangleInfo.texel_coordinates.y;
        float cutoff = location.y + rectangleInfo.size.y / 2.0;
        
        
        // if(boundary > cutoff) {
        float edgeSoftness  = 1.0f;
        // Calculate distance to edge.   
        float dist = box_signed_distance_field_rounding(rectangleInfo.texel_coordinates.xy - location - (rectangleInfo.size/2.0f), rectangleInfo.size / 2.0f, radius);
            // Smooth the result (free antialiasing).
        float smoothedAlpha =  1.0 - smoothstep(0.0f, edgeSoftness * 2.0 ,dist);
            // This will be our resulting "shape". 
        vec4 quadColor = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(chosen_color.rgb, smoothedAlpha), smoothedAlpha);
        FragColor = quadColor;
    } else {
        FragColor = chosen_color;
    }    
}