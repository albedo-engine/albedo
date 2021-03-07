#version 450

layout( location = 0 ) in vec2 vUv;

layout( set = 0, binding = 0 ) uniform sampler uTextureSampler;
layout( set = 0, binding = 1 ) uniform texture2D uTexture;

layout(location = 0) out vec4 outColor;

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
  // outColor = texture(sampler2D(uTexture, uTextureSampler), vUv).rgba
  //   / float(RenderSettings.frameCount);
  // outColor.rgb = ACESFilmTonemapping(outColor.rgb);

  outColor.rgb = texture(sampler2D(uTexture, uTextureSampler), vUv).rgb;
  outColor.a = 1.0;

  // outColor.rgb = vec3(1.0, 0.0, 0.0);
  // outColor.a = 1.0;
}