// Umbriel Shaders starter — focus-ring-pulse
vec4 border(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    float ring = smoothstep(0.014, 0.0, edge);
    float pulse = 0.55 + 0.45 * sin(umbriel_time * 3.0);
    vec3 tint = vec3(0.85, 1.0, 0.0);
    float amount = ring * pulse;
    return vec4(mix(base.rgb, tint, amount), max(base.a, amount));
}
