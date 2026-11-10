// Umbriel Shaders starter — scanline-memory
vec4 screen(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float lines = 0.86 + 0.14 * sin(uv.y * umbriel_size.y * 0.6 + umbriel_time * 4.0);
    return vec4(base.rgb * lines, base.a);
}
