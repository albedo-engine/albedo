#ifndef TEXTURE_UTILS_H
#define TEXTURE_UTILS_H

void
fetchBounds(uint textureIndex, out vec4 bounds, out float layer)
{
  uvec4 data = texelFetch(usampler1D(textureInfo, samplerNearest), int(textureIndex), 0);
  layer = float((data.w & 0xFF000000) >> 24);
  bounds = vec4(float(data.x), float(data.y), float(data.z), float(data.w & 0x00FFFFFF));
}

vec4
fetchTexture(uint textureIndex, vec2 uv)
{
  uv = mod(uv, vec2(1.0, 1.0));
  // @todo: optimize away.
  vec2 atlasSize = vec2(textureSize(textureAtlas, 0).xy);
  float layer = 0.0;
  vec4 bounds = vec4(0.0);
  fetchBounds(textureIndex, bounds, layer);
  bounds.xy /= atlasSize;
  bounds.zw /= atlasSize;
  // linear sampling
  return textureLod(
    sampler2DArray(textureAtlas, samplerNearest),
    vec3(bounds.xy + (uv * bounds.zw), layer),
    0.0
  );
}

#endif
