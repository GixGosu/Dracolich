#version 330 core

// Input vertex attributes
layout(location = 0) in vec3 a_Position;    // Vertex position (skybox cube or dome)

// Uniforms
uniform mat4 u_View;            // View matrix (camera rotation only, no translation)
uniform mat4 u_Projection;      // Projection matrix

// Output to fragment shader
out vec3 v_Position;            // Position for gradient calculation
out vec3 v_ViewDir;             // View direction for sun/moon positioning

void main() {
    // Pass position for gradient calculation
    v_Position = a_Position;
    v_ViewDir = a_Position;

    // Transform to clip space
    // We remove translation from view matrix to keep sky centered on camera
    vec4 pos = u_Projection * u_View * vec4(a_Position, 1.0);

    // Set z to w so that the sky is always at maximum depth (z/w = 1.0)
    gl_Position = pos.xyww;
}
