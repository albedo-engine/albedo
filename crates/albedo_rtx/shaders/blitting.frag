#version 450

#extension GL_EXT_samplerless_texture_functions : enable

// @todo: split global uniforms.
#include "imports/structures.glsl"
#include "imports/colorspace.glsl"

layout( location = 0 ) in vec2 vUv;

layout( set = 0, binding = 0 ) uniform sampler uTextureSampler;
layout( set = 0, binding = 1 ) uniform texture2D uTexture;
layout (set = 0, binding = 2) uniform GlobalUniformBuffer {
  GlobalUniforms global;
};

layout(location = 0) out vec4 outColor;

void main() {
  vec2 uv = vUv * vec2(global.dimensions) / vec2(textureSize(uTexture, 0));
  outColor = texture(sampler2D(uTexture, uTextureSampler), uv).rgba / float(global.frame);
  outColor.rgb = ACESFilmTonemapping(outColor.rgb);
  outColor.rgb = linearTosRGB(outColor.rgb);
  outColor.a = 1.0;
}
