#version 330 core

// Input from vertex shader
in vec2 v_TexCoord;         // Texture coordinates
in float v_Light;           // Light level (0.0 to 1.0)
in float v_FogDistance;     // Distance from camera
in vec3 v_Normal;           // Normal vector

// Uniforms
uniform sampler2D u_TextureAtlas;   // Block texture atlas
uniform float u_TimeOfDay;          // Time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
uniform vec3 u_FogColor;            // Sky/fog color
uniform float u_FogStart;           // Distance where fog starts
uniform float u_FogEnd;             // Distance where fog is fully opaque
uniform float u_RenderDistance;     // Maximum render distance

// Output
out vec4 FragColor;

void main() {
    // Sample the texture atlas
    vec4 texColor = texture(u_TextureAtlas, v_TexCoord);

    // Discard fully transparent fragments
    if (texColor.a < 0.1) {
        discard;
    }

    // Calculate day/night lighting multiplier
    // At noon (0.5): full brightness
    // At midnight (0.0 or 1.0): darker
    float dayNightCycle = 1.0 - abs(u_TimeOfDay - 0.5) * 2.0; // 0.0 at midnight, 1.0 at noon
    float ambientLight = mix(0.15, 1.0, dayNightCycle);       // Min 15% brightness at night

    // Apply block light level (from torches, etc.) and ambient light
    float totalLight = max(v_Light, ambientLight);

    // Apply lighting to texture color
    vec3 litColor = texColor.rgb * totalLight;

    // Calculate fog factor (linear fog)
    float fogFactor = clamp((v_FogDistance - u_FogStart) / (u_FogEnd - u_FogStart), 0.0, 1.0);

    // Mix the lit color with fog color based on distance
    vec3 finalColor = mix(litColor, u_FogColor, fogFactor);

    // Apply day/night tint - make things slightly blue at night, warm during day
    vec3 dayTint = vec3(1.0, 0.98, 0.92);    // Slight warm tint during day
    vec3 nightTint = vec3(0.7, 0.75, 1.0);   // Slight blue tint at night
    vec3 timeTint = mix(nightTint, dayTint, dayNightCycle);
    finalColor *= timeTint;

    FragColor = vec4(finalColor, texColor.a);
}
