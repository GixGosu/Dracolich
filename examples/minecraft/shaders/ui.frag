#version 330 core

// Input from vertex shader
in vec2 v_TexCoord;         // Texture coordinates
in vec4 v_Color;            // Vertex color

// Uniforms
uniform sampler2D u_Texture;    // UI texture (icons, text, etc.)
uniform int u_UseTexture;       // Flag: 1 = sample texture, 0 = use vertex color only
uniform float u_Alpha;          // Global alpha multiplier for fading UI elements

// Output
out vec4 FragColor;

void main() {
    vec4 finalColor;

    if (u_UseTexture == 1) {
        // Sample texture and multiply by vertex color
        vec4 texColor = texture(u_Texture, v_TexCoord);
        finalColor = texColor * v_Color;
    } else {
        // Use vertex color only (for solid colored UI elements)
        finalColor = v_Color;
    }

    // Apply global alpha
    finalColor.a *= u_Alpha;

    // Discard fully transparent fragments
    if (finalColor.a < 0.01) {
        discard;
    }

    FragColor = finalColor;
}
