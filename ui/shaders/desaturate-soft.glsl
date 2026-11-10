// Umbriel Shaders starter — desaturate-soft
vec4 window(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float grey = dot(base.rgb, vec3(0.299, 0.587, 0.114));
    float edge = smoothstep(0.0, 0.65, length(uv - 0.5));
    return vec4(mix(base.rgb, vec3(grey), edge * 0.72), base.a);
}
