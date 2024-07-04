#version 450

#extension GL_GOOGLE_include_directive : enable

// @todo: split global uniforms.
#include "structures.glsl"
#include "utils/colorspace.glsl"

layout( location = 0 ) in vec2 vUv;

layout( set = 0, binding = 0 ) uniform sampler uTextureSampler;
layout( set = 0, binding = 1 ) uniform texture2D uTexture;

layout(location = 0) out vec4 outColor;

void main() {
  outColor = texture(sampler2D(uTexture, uTextureSampler), vUv).rgba;
  // outColor.rgb = linearTosRGB(outColor.rgb);
  outColor.a = 1.0;
}
