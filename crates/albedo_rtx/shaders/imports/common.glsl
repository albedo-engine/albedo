#ifndef COMMON_H
#define COMMON_H

#define EPSILON 0.00000001
#define NEG_EPSILON -0.00000001
#define EPSILON1 1.0001

// +37 because WGSL has a different maximum value.
#define MAX_FLOAT 3.402823466e+37

#define MAX_UINT 0xFFFFFFFF
#define INVALID_UINT MAX_UINT

/**
 * Math
 */

vec3
transformPosition(vec3 position, mat4 transform)
{
  vec4 pos = transform * vec4(position, 1.0);
  return pos.xyz / pos.w;
}

vec3
transformDirection(vec3 direction, mat4 transform)
{
  return (transform * vec4(direction, 0.0)).xyz;
}

vec3
project(vec3 val, const vec3 normal, const vec3 tangent, const vec3 bitangent)
{
  return tangent * val.x + bitangent * val.y + normal * val.z;
}

#endif // COMMON_H
