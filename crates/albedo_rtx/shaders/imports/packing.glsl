struct GBufferSample {
  vec3 albedo;
  vec3 normal;
  float depth;
  uint id;
};

vec3
unpackOctahedral(uint enc)
{
    // Decode RG16_UNORM
    uvec2 u = uvec2(enc & 0xffffu, enc >> 16);
    vec2 p = vec2(u) / float(0xffff);

    // Convert to [-1..1]
    p = p * 2.0 - 1.0;

    // Decode the octahedron
    // https://twitter.com/Stubbesaurus/status/937994790553227264
    vec3 n = vec3(p.x, p.y, 1.0 - abs(p.x) - abs(p.y));
    float t = max(0, -n.z);
    n.xy += mix(vec2(t), vec2(-t), greaterThanEqual(n.xy, vec2(0)));

    return normalize(n);
}

uint
packOctahedral(vec3 normal)
{
    // Project the sphere onto the octahedron (|x|+|y|+|z| = 1) and then onto the xy-plane
    float invL1Norm = 1.0 / (abs(normal.x) + abs(normal.y) + abs(normal.z));
    vec2 p = normal.xy * invL1Norm;

    // Wrap the octahedral faces from the negative-Z space
    p = (normal.z < 0) ? (1.0 - abs(p.yx)) * mix(vec2(-1.0), vec2(1.0), greaterThanEqual(p.xy, vec2(0))) : p;

    // Convert to [0..1]
    p = clamp(p.xy * 0.5 + 0.5, vec2(0), vec2(1));

    // Encode as RG16_UNORM
    uvec2 u = uvec2(p * 0xffffu);
    return u.x | (u.y << 16);
}

uint
packRGBE(vec3 v)
{
    vec3 va = max(vec3(0), v);
    float max_abs = max(va.r, max(va.g, va.b));
    if(max_abs == 0)
        return 0u;

    float exponent = floor(log2(max_abs));

    uint result;
    result = uint(clamp(exponent + 20, 0, 31)) << 27;

    float scale = pow(2, -exponent) * 256.0;
    uvec3 vu = min(uvec3(511), uvec3(round(va * scale)));
    result |= vu.r;
    result |= vu.g << 9;
    result |= vu.b << 18;

    return result;
}

vec3
unpackRGBE(uint x)
{
    int exponent = int(x >> 27) - 20;
    float scale = pow(2, exponent) / 256.0;

    vec3 v;
    v.r = float(x & 0x1ff) * scale;
    v.g = float((x >> 9) & 0x1ff) * scale;
    v.b = float((x >> 18) & 0x1ff) * scale;

    return v;
}

uvec4
packGbuffer(vec3 normal, float dist, vec3 albedo, uint id) {
  return uvec4(packRGBE(albedo), packOctahedral(normal), floatBitsToUint(dist), id);
}

GBufferSample
unpackGbuffer(uvec4 data)
{
  GBufferSample s;
  s.albedo = unpackRGBE(data.r);
  s.normal = unpackOctahedral(data.g);
  s.depth = uintBitsToFloat(data.b);
  s.id = data.a;
  return s;
}
