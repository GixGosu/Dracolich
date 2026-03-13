#version 330 core

// Uniforms
uniform vec4 u_Color;           // Highlight color (typically white or black with alpha)
uniform float u_Time;           // Time for pulsing animation (optional)

// Output
out vec4 FragColor;

void main() {
    // Apply pulsing alpha animation for visual feedback
    float pulse = sin(u_Time * 3.0) * 0.1 + 0.9; // Oscillate between 0.8 and 1.0

    vec4 highlightColor = u_Color;
    highlightColor.a *= pulse;

    FragColor = highlightColor;
}
