// Umbriel Shaders starter — panel-glide
vec4 animation(vec2 uv) {
    float p = umbriel_clamped_progress;
    vec2 shifted = vec2(uv.x - (1.0 - p) * 0.12, uv.y);
    return umbriel_sample(shifted);
}
