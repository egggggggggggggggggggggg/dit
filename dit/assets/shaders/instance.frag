#version 450
layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 fg;
layout(location = 2) in vec3 bg;

layout(location = 0) out vec4 uFragColor;

float median(float a, float b, float c) {
    return max(min(a, b), min(max(a, b), c));
}

void main() {
    vec3 s = texture(texSampler, uv).rgb;
    float d = median(s.r, s.g, s.b) - 0.5;
    float w = clamp(d / fwidth(d) + 0.5, 0.0, 1.0);
    uFragColor = mix(vec4(bg, 0.0), vec4(fg, 0.0), w);
}
