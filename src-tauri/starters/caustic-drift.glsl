// Umbriel Shaders starter — caustic-drift
vec4 window(vec2 uv) {
    float t = umbriel_time;
    vec2 drift = vec2(sin(uv.y * 14.0 + t * 1.3), cos(uv.x * 12.0 + t * 1.1)) * 0.004;
    return umbriel_sample(uv + drift);
}
