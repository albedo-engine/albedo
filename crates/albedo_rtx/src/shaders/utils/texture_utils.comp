#ifndef TEXTURE_UTILS_H
#define TEXTURE_UTILS_H

void
fetchBounds(uint textureIndex, out vec4 bounds, out float layer)
{
  uvec4 data = texelFetch(usampler1D(textureInfo, samplerNearest), int(textureIndex), 0).rgba;
  layer = float((data.w & 0xFF000000) >> 24);
  bounds = vec4(float(data.x), float(data.y), float(data.z), float(data.w & 0x00FFFFFF));
}

vec4
fetchTexture(sampler samp, uint textureIndex, vec2 uv)
{
  uv = mod(uv, vec2(1.0, 1.0));
  // @todo: optimize away.
  vec2 atlasSize = vec2(textureSize(textureAtlas, 0).xy);
  float layer = 0.0;
  vec4 bounds = vec4(0.0);
  fetchBounds(textureIndex, bounds, layer);
  bounds.xy /= atlasSize;
  bounds.zw /= atlasSize;
  return texture(
    sampler2DArray(textureAtlas, samp),
    vec3(bounds.xy + (uv * bounds.zw), layer)
  );
}

#endif // TEXTURE_UTILS_H
