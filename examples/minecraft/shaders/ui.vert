#version 330 core

// Input vertex attributes
layout(location = 0) in vec2 a_Position;    // 2D position in screen space (pixels)
layout(location = 1) in vec2 a_TexCoord;    // Texture coordinates
layout(location = 2) in vec4 a_Color;       // Vertex color (for colored UI elements)

// Uniforms
uniform mat4 u_Projection;      // Orthographic projection matrix (screen space)

// Output to fragment shader
out vec2 v_TexCoord;            // Pass through texture coordinates
out vec4 v_Color;               // Pass through vertex color

void main() {
    // Transform 2D position to clip space using orthographic projection
    gl_Position = u_Projection * vec4(a_Position, 0.0, 1.0);

    // Pass through attributes
    v_TexCoord = a_TexCoord;
    v_Color = a_Color;
}
