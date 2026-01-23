#version 450
layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 o_uv;

layout(location = 0) out vec4 uFragColor;
const vec4 outsideColor = vec4(1.0, 1.0, 1.0, 0.0);
const vec4 insideColor = vec4(0.0, 0.0, 0.0, 0.0);
float median(float a, float b, float c) {
   return max(min(a, b), min(max(a, b), c));
}

void main() {
   vec3 s = texture(texSampler, o_uv).rgb;
   float d = median(s.r, s.g, s.b) - 0.5;
   float w = clamp(d / fwidth(d) + 0.5, 0.0, 1.0);
   uFragColor = mix(outsideColor, insideColor, w);
}
