#ifndef STRUCTS_H
#define STRUCTS_H

struct GlobalUniforms
{
  uint frame;
  uint seed;
  uint bounces;
  uint padding;
  uvec2 dimensions;
};

struct BVHNode {
  vec4 n0;
  vec4 n1;
  vec4 n2;
  vec4 n3;
  vec4 n4;
};

struct Instance
{
  // @todo: reduce size of this struct.
  mat4 modelToWorld;
  mat4 worldToModel;
  uint materialIndex;
  uint bvhRootIndex;
  uint vertexRootIndex;
  uint primitiveRootIndex;
};

struct Vertex
{
  vec4 position;
  vec4 normal;
};

struct Light
{
  vec4 normal;
  vec4 tangent;
  vec4 bitangent;
  float intensity;
  float padding_0;
  float padding_1;
  float padding_2;
};

/**
 * - `throughput` saved in `origin.w`, `dir.w`, `radiance,w`
 */
struct RayPayload {
  vec4 origin;
  vec4 dir;
  vec4 radiance;
  uvec4 terminated;
};

struct Ray {
  vec3 origin;
  vec3 dir;
};

struct Intersection {
  vec2 uv;
  uint index;
  uint instance;
  uint materialIndex;
  uint emitter;
  float dist;
  float padding_1; // not needed I think
};

#endif // STRUCTS_H
