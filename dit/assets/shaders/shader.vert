#version 450
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 o_uv;
const vec2 screenSize = vec2(1920.0, 1080.0);
void main() {
   o_uv = uv;
   vec2 normalized = aPos / screenSize;
   vec2 ndc = normalized * 2.0 - 1.0;
   ndc.y = -ndc.y;

   gl_Position = vec4(ndc, 0.0, 1.0);
}
