#version 450

#extension GL_EXT_samplerless_texture_functions : enable
#extension GL_GOOGLE_include_directive : enable

layout(location = 0) in vec2 vUv;

layout(push_constant) uniform pushConstants {
  uvec2 size;
} constants;

layout(set = 0, binding = 0) uniform sampler uTextureSampler;
layout(set = 0, binding = 1) uniform texture2D uTexture;

layout(location = 0) out vec4 outColor;

void main() {
  vec2 t = vec2(constants.size) / vec2(textureSize(uTexture, 0));
  outColor = texture(sampler2D(uTexture, uTextureSampler), vUv * t).rgba;
  outColor.a = 1.0;
}
