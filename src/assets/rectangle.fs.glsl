#version 430 core
out vec4 FragColor;
// from http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
float box_signed_distance_field_rounding(vec2 CenterPosition, vec2 Size, float Radius) {
    return length(max(abs(CenterPosition)-Size+Radius,0.0))-Radius;
}

uniform float radius;
uniform vec2 rect_pos;
uniform sampler2D texture_sampler;

// *much* cleaner now.
in RectangleInfo {
    vec2 tex_coords;
    vec2 size;
    vec4 color;
    float use_texture;
} rectangleInfo;

const float smoothness = 0.7;

void main()
{

    vec4 chosen_color = rectangleInfo.color;
    if(rectangleInfo.use_texture == 0.0) {
        chosen_color = rectangleInfo.color;
    } else {
        chosen_color = texture(texture_sampler, rectangleInfo.tex_coords);
    }
    
    if(radius > 0.0) {
        // The pixel space scale of the rectangle.       
        vec2 location = rect_pos;
        float cutoff = location.y + rectangleInfo.size.y / 2.0;
        float edgeSoftness  = 0.1f;
        // the actual magic behind signed distance fields/functions
        float dist = box_signed_distance_field_rounding(gl_FragCoord.xy - location - (rectangleInfo.size/2.0f), rectangleInfo.size / 2.0f, radius);
        float smoothedAlpha =  1.0 - smoothstep(0.0f, edgeSoftness * 2.0 ,dist);
        // This will be our resulting "shape". 
        vec4 quadColor = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(chosen_color.rgb, smoothedAlpha), smoothedAlpha);
        FragColor = quadColor;
    } else {
        FragColor = chosen_color;
    }    
}