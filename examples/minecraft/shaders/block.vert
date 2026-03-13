#version 330 core

// Input vertex attributes
layout(location = 0) in vec3 a_Position;    // Vertex position in model space
layout(location = 1) in vec2 a_TexCoord;    // Texture coordinates for atlas sampling
layout(location = 2) in vec3 a_Normal;      // Normal vector for lighting
layout(location = 3) in float a_Light;      // Per-vertex light level (0.0 to 1.0)

// Uniforms
uniform mat4 u_Model;           // Model matrix (chunk position offset)
uniform mat4 u_View;            // View matrix (camera transformation)
uniform mat4 u_Projection;      // Projection matrix (perspective)
uniform vec3 u_CameraPos;       // Camera position in world space for fog calculation

// Output to fragment shader
out vec2 v_TexCoord;            // Pass through texture coordinates
out float v_Light;              // Pass through light level
out float v_FogDistance;        // Distance from camera for fog calculation
out vec3 v_Normal;              // Pass through normal for potential lighting

void main() {
    // Transform vertex position to world space
    vec4 worldPos = u_Model * vec4(a_Position, 1.0);

    // Calculate distance from camera for fog
    v_FogDistance = length(worldPos.xyz - u_CameraPos);

    // Transform to clip space
    gl_Position = u_Projection * u_View * worldPos;

    // Pass through attributes to fragment shader
    v_TexCoord = a_TexCoord;
    v_Light = a_Light;
    v_Normal = mat3(transpose(inverse(u_Model))) * a_Normal;
}
