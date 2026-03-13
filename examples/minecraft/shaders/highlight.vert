#version 330 core

// Input vertex attributes
layout(location = 0) in vec3 a_Position;    // Vertex position (cube outline)

// Uniforms
uniform mat4 u_Model;           // Model matrix (position of highlighted block)
uniform mat4 u_View;            // View matrix (camera transformation)
uniform mat4 u_Projection;      // Projection matrix (perspective)

void main() {
    // Transform vertex to clip space
    gl_Position = u_Projection * u_View * u_Model * vec4(a_Position, 1.0);
}
