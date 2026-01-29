#version 450
layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 o_uv;
layout(location = 0) out vec4 uFragColor;

const vec4 fgColor = vec4(1.0, 1.0, 1.0, 0.0);
const vec4 bgColor = vec4(0.0, 0.0, 0.0, 0.0);
float median(float a, float b, float c) {
   return max(min(a, b), min(max(a, b), c));
}
float screenPxRange() {
   vec2 unitRange = vec2(2.0) / textureSize(texSampler, 0);
   vec2 screenTexSize = vec2(1.0) / fwidth(o_uv);
   return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

void main() {
   vec3 msd = texture(texSampler, o_uv).rgb;
   float sd = median(msd.r, msd.g, msd.b);
   float screenPxDistance = screenPxRange() * (sd - 0.5);
   float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
   uFragColor = mix(bgColor, fgColor, opacity);
}
