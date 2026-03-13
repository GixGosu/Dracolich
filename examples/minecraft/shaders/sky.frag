#version 330 core

// Input from vertex shader
in vec3 v_Position;         // Position for gradient calculation
in vec3 v_ViewDir;          // View direction

// Uniforms
uniform float u_TimeOfDay;          // Time of day (0.0 to 1.0)
uniform vec3 u_SunDirection;        // Direction to the sun in world space
uniform vec3 u_MoonDirection;       // Direction to the moon in world space

// Output
out vec4 FragColor;

void main() {
    // Normalize view direction
    vec3 viewDir = normalize(v_ViewDir);

    // Calculate day/night cycle value (0.0 = night, 1.0 = day)
    float dayNightCycle = 1.0 - abs(u_TimeOfDay - 0.5) * 2.0;

    // Define sky colors for different times of day
    vec3 skyColorDay = vec3(0.53, 0.81, 0.92);      // Day sky blue
    vec3 horizonColorDay = vec3(0.8, 0.9, 1.0);     // Lighter blue near horizon
    vec3 skyColorNight = vec3(0.01, 0.01, 0.05);    // Dark blue/black night
    vec3 horizonColorNight = vec3(0.05, 0.05, 0.15); // Slightly lighter near horizon

    // Sunrise/sunset colors
    vec3 sunsetColor = vec3(1.0, 0.5, 0.3);         // Orange/red sunset
    float sunsetFactor = 0.0;

    // Detect sunrise/sunset (around 0.25 and 0.75 time of day)
    if (u_TimeOfDay > 0.2 && u_TimeOfDay < 0.3) {
        sunsetFactor = 1.0 - abs(u_TimeOfDay - 0.25) * 10.0; // Peak at 0.25
    } else if (u_TimeOfDay > 0.7 && u_TimeOfDay < 0.8) {
        sunsetFactor = 1.0 - abs(u_TimeOfDay - 0.75) * 10.0; // Peak at 0.75
    }

    // Calculate gradient based on vertical position (-1 to 1)
    float horizonBlend = pow(1.0 - abs(viewDir.y), 2.0); // More color near horizon

    // Interpolate between day and night colors
    vec3 skyColor = mix(skyColorNight, skyColorDay, dayNightCycle);
    vec3 horizonColor = mix(horizonColorNight, horizonColorDay, dayNightCycle);

    // Apply sunset/sunrise tint near horizon
    horizonColor = mix(horizonColor, sunsetColor, sunsetFactor * horizonBlend);

    // Blend sky and horizon colors based on vertical view angle
    vec3 finalColor = mix(skyColor, horizonColor, horizonBlend);

    // --- Render Sun ---
    vec3 sunDir = normalize(u_SunDirection);
    float sunDot = dot(viewDir, sunDir);
    float sunIntensity = pow(max(sunDot, 0.0), 512.0); // Sharp sun disk
    float sunGlow = pow(max(sunDot, 0.0), 8.0) * 0.3;  // Soft glow around sun

    vec3 sunColor = vec3(1.0, 1.0, 0.9);
    if (dayNightCycle > 0.3) { // Only show sun during day
        finalColor += sunColor * (sunIntensity + sunGlow);
    }

    // --- Render Moon ---
    vec3 moonDir = normalize(u_MoonDirection);
    float moonDot = dot(viewDir, moonDir);
    float moonIntensity = pow(max(moonDot, 0.0), 512.0); // Sharp moon disk
    float moonGlow = pow(max(moonDot, 0.0), 16.0) * 0.1; // Subtle glow

    vec3 moonColor = vec3(0.8, 0.8, 0.9);
    if (dayNightCycle < 0.7) { // Only show moon during night
        finalColor += moonColor * (moonIntensity * 0.8 + moonGlow);
    }

    // Add subtle stars at night
    if (dayNightCycle < 0.3) {
        // Simple procedural stars using position-based noise
        float starNoise = fract(sin(dot(viewDir.xy, vec2(12.9898, 78.233))) * 43758.5453);
        if (starNoise > 0.998 && viewDir.y > 0.0) { // Only above horizon
            float starBrightness = (1.0 - dayNightCycle) * 0.8;
            finalColor += vec3(starBrightness);
        }
    }

    FragColor = vec4(finalColor, 1.0);
}
