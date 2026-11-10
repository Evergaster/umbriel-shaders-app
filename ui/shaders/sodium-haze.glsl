// Umbriel Shaders starter — sodium-haze
vec4 cursor(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float dist = length((uv - umbriel_pointer) * umbriel_size);
    float reach = 0.45 * min(umbriel_size.x, umbriel_size.y);
    float falloff = 1.0 - smoothstep(0.0, reach, dist);
    vec3 tint = vec3(1.0, 0.62, 0.10);
    float amount = falloff * falloff * 0.42;
    return vec4(mix(base.rgb, tint, amount), max(base.a, amount));
}
