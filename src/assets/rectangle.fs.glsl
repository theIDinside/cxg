#version 430 core
out vec4 FragColor;
// The RGB Color
in vec4 rect_color;
// The interpolation value; decides if we use the bound texture
in float interpolation;
/// size of the rectangle currently getting drawn
in vec2 u_size;

// actual fragment position in the world, used for the SDF calculation
in vec2 texel_coords;

// the texture coordinates
in vec2 uv_coords;
// radius of the corners
uniform float radius;
uniform vec2 rect_pos;
uniform sampler2D texture_sampler;
const float smoothness = 0.7;

// from http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
float box_signed_distance_field_rounding(vec2 CenterPosition, vec2 Size, float Radius) {
    return length(max(abs(CenterPosition)-Size+Radius,0.0))-Radius;
}

void main()
{
    float interpolation = rect_color.w;
    vec3 rect_color2 = rect_color.xyz;
    vec4 chosen_color = vec4(0.0, 0.0, 1.0, 0.0);

    if(interpolation == 0.0) {
        chosen_color = vec4(rect_color2, 1.0);
    } else {
        vec4 sampledTexture = texture(texture_sampler, uv_coords);
        chosen_color = 
        mix(sampledTexture, vec4(rect_color2, 1.0), 0.005);
    }
    
    if(radius > 0.0) {
        // The pixel space scale of the rectangle.
        vec2 size = u_size;
        
        // the pixel space location of the rectangle.
        vec2 location = rect_pos;
        float boundary = texel_coords.y;
        float cutoff = location.y + size.y / 2.0;
        
        
        // if(boundary > cutoff) {
            float edgeSoftness  = 1.0f;
            // Calculate distance to edge.   
            float dist = box_signed_distance_field_rounding(texel_coords.xy - location - (size/2.0f), size / 2.0f, radius);
            // Smooth the result (free antialiasing).
            float smoothedAlpha =  1.0 - smoothstep(0.0f, edgeSoftness * 2.0 ,dist);
            // This will be our resulting "shape". 
            vec4 quadColor = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(chosen_color.rgb, smoothedAlpha), smoothedAlpha);
            FragColor = quadColor;
        /*
        } else {
            FragColor = chosen_color;
        }*/
    } else {
        FragColor = chosen_color;
    }    
}