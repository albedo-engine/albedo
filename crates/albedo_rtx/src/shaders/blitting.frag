#version 450

// @todo: split global uniforms.
#include "structures.comp"

layout( location = 0 ) in vec2 vUv;

layout( set = 0, binding = 0 ) uniform sampler uTextureSampler;
layout( set = 0, binding = 1 ) uniform texture2D uTexture;
layout (set = 0, binding = 2) uniform GlobalUniformBuffer {
  GlobalUniforms global;
};

layout(location = 0) out vec4 outColor;

vec4
LinearToSRGB(vec4 linearRGB)
{
    bvec3 cutoff = lessThan(linearRGB.rgb, vec3(0.0031308));
    vec3 higher = vec3(1.055)*pow(linearRGB.rgb, vec3(1.0/2.4)) - vec3(0.055);
    vec3 lower = linearRGB.rgb * vec3(12.92);
    return vec4(mix(higher, lower, cutoff), linearRGB.a);
}

// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 ACESFilmTonemapping(vec3 x)
{
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x + b)) / (x*(c*x + d) + e), 0.0f, 1.0f);
}

void main() {
  outColor = texture(sampler2D(uTexture, uTextureSampler), vUv).rgba / float(global.frame);
  // outColor.rgb = ACESFilmTonemapping(outColor.rgb);
  // outColor = LinearToSRGB(outColor);
  outColor.a = 1.0;
}
