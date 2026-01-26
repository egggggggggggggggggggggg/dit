#version 450
layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inUV;

layout(location = 2) in vec2 instPos;
layout(location = 3) in vec2 instUv;
layout(location = 4) in vec3 instFg;
layout(location = 5) in vec3 instBg;

layout(location = 0) out vec2 o_uv;
layout(location = 1) out vec3 o_fg;
layout(location = 2) out vec3 o_bg;
void main() {
    o_uv = instUv;
    o_fg = instFg;
    o_bg = instBg;
    gl_Position = vec4((inPos.x + instPos.x) * instSize.x, (inPos.y + instPos.y) * instSize.y, 0.0, 1.0);
}
