// Umbriel Shaders starter — paper-fade
vec4 animation(vec2 uv) {
    float p = umbriel_clamped_progress;
    vec2 shifted = uv + vec2(0.0, (1.0 - p) * 0.02);
    vec4 base = umbriel_sample(shifted);
    vec3 paper = vec3(0.055, 0.055, 0.050);
    return vec4(mix(base.rgb, paper, p * 0.45), base.a);
}
