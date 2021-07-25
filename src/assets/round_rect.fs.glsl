#version 430 core
out vec4 FragColor;
in vec4 rect_color;
/// size of the rectangle currently getting drawn
in vec2 u_size;

in vec2 texel_coords;

// size of the window we're drawing to
uniform float win_width;
uniform float win_height;
// radius of the corners
uniform float radius;
uniform vec2 rect_pos;

const float smoothness = 0.7;

// from http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
float roundedBoxSDF(vec2 CenterPosition, vec2 Size, float Radius) {
    return length(max(abs(CenterPosition)-Size+Radius,0.0))-Radius;
}

void main()
{
    vec2 p = vec2(win_width, win_height);
    if(radius > 0.0) {
        // The pixel space scale of the rectangle.
        vec2 size = u_size;
        // the pixel space location of the rectangle.
        vec2 location = rect_pos;

        // How soft the edges should be (in pixels). Higher values could be used to simulate a drop shadow.
        float edgeSoftness  = 1.0f;
        
        // The radius of the corners (in pixels).
        
        // Calculate distance to edge.   
        float dist 		= roundedBoxSDF(texel_coords.xy - location - (size/2.0f), size / 2.0f, radius);
        
        // Smooth the result (free antialiasing).
        float smoothedAlpha =  1.0f-smoothstep(0.0f, edgeSoftness * 2.0f,dist);
        
        // Return the resultant shape.
        vec4 quadColor		= mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(rect_color.rgb, smoothedAlpha), smoothedAlpha);
    
        FragColor 			 = quadColor;
        // FragColor 			 = mix(quadColor, shadowColor, shadowAlpha - smoothedAlpha);
    } else {
        FragColor = rect_color;
    }    
}