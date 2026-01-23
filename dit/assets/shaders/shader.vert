#version 450
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 o_uv;
void main() {
   o_uv = uv;
   gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
}
