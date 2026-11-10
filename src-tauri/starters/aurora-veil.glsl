// Umbriel Shaders starter — aurora-veil
vec4 animation(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float p = umbriel_clamped_progress;
    float t = umbriel_time * 0.55;
    float band = sin(uv.x * 9.0 + t + uv.y * 3.5);
    float veil = smoothstep(0.10, 0.95, band * 0.5 + 0.5);
    vec3 tint = mix(vec3(0.25, 1.0, 0.55), vec3(0.35, 0.65, 1.0), uv.y);
    float amount = veil * (0.55 * (1.0 - p) + 0.18);
    return vec4(base.rgb + tint * amount, base.a);
}
