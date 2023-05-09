#version 450

layout(location=0) in vec2 uv;

layout(location=0) out vec2 vUv;

void main() {
    vec2 pos = uv * 2.0 - 1.0;
    gl_Position = vec4(pos, 0.0, 1.0);
}
