struct TextureInfo
{
  uint x;
  uint y;
  uint width;
  uint layerAndHeight; // 24 bits for height, 8 bits for layer index.
};

struct Material
{
  vec4  color;
  float roughnessFactor;
  float metallic;
  uint  albedoTexture;
  // @todo: for now, metal in B channel and roughness in G.
  uint  mraTexture;
};
