#ifndef INTERSECTION_UTILS_H
#define INTERSECTION_UTILS_H

#include "common.glsl"

struct Primitive
{
  Vertex v0;
  Vertex v1;
  Vertex v2;
};

Vertex
getVertex(uint vertexOffset, uint index)
{
  return vertices[vertexOffset + indices[index]];
}

/**
 * Retrieve the triangle from vertex offset and index
 *
 * @param instance Instance to extract the primitive from
 * @param intersection Intersection data
 */
Primitive extractPrimitive(Instance instance, Intersection intersection)
{
  Primitive p;
  p.v0 = getVertex(instance.vertexRootIndex, intersection.index);
  p.v1 = getVertex(instance.vertexRootIndex, intersection.index + 1);
  p.v2 = getVertex(instance.vertexRootIndex, intersection.index + 2);
  return p;
}

vec2 interpolateBarycentric(vec2 v0, vec2 v1, vec2 v2, vec3 barycentric)
{
  return (
    barycentric.x * v0 +
    barycentric.y * v1 +
    barycentric.z * v2
  );
}
vec3 interpolateBarycentric(vec3 v0, vec3 v1, vec3 v2, vec3 barycentric)
{
  return (
    barycentric.x * v0 +
    barycentric.y * v1 +
    barycentric.z * v2
  );
}

vec3 barycentricCoordinates(vec2 uv)
{
  return vec3(1.0 - uv.x - uv.y, uv);
}

float
intersectPlane(Ray ray, vec3 normal, vec3 origin, vec3 edge01, vec3 edge02)
{
  float NdotD = dot(normal, ray.dir);
  if (NdotD < EPSILON) { return MAX_FLOAT; }

  float t = dot(normal, origin - ray.origin) / NdotD;
  if (t < EPSILON) { return MAX_FLOAT; }

  vec3 intersection = (ray.origin + ray.dir * t) - origin;

  // Check not before first edge.
  float interProj = dot(edge01, intersection);
  if (interProj < EPSILON || interProj > dot(edge01, edge01)) { return MAX_FLOAT; }

  interProj = dot(edge02, intersection);
  if (interProj < EPSILON || interProj > dot(edge02, edge02)) { return MAX_FLOAT; }

  return t;
}

float
intersectAABB(vec3 origin, vec3 inverseDir, vec3 aabbMin, vec3 aabbMax)
{
  // Ray is assumed to be in local coordinates, ie:
  // ray = inverse(objectMatrix * invCameraMatrix) * ray

  // Equation of ray: O + D * t

  vec3 tbottom = inverseDir * (aabbMin - origin);
  vec3 ttop = inverseDir * (aabbMax - origin);

  vec3 tmin = min(ttop, tbottom);
  vec3 tmax = max(ttop, tbottom);

  float smallestMax = min(min(tmax.x, tmax.y), min(tmax.x, tmax.z));
  float largestMin = max(max(tmin.x, tmin.y), max(tmin.x, tmin.z));

  if (smallestMax < largestMin || smallestMax < 0.0) { return MAX_FLOAT; }
  return (largestMin > 0.0) ? largestMin : smallestMax;
}

bool
isIntersectingAABB(vec3 origin, vec3 inverseDir, vec3 boxMin, vec3 boxMax)
{
  // Ray is assumed to be in local coordinates, ie:
  // ray = inverse(objectMatrix * invCameraMatrix) * ray
  // Equation of ray: O + D * t
  vec3 tMin = (boxMin - origin) * inverseDir;
  vec3 tMax = (boxMax - origin) * inverseDir;
  vec3 t1 = min(tMin, tMax);
  vec3 t2 = max(tMin, tMax);
  float tNear = max(max(t1.x, t1.y), t1.z);
  float tFar = min(min(t2.x, t2.y), t2.z);
  return tFar > tNear;
}

float
intersectAABB(Ray ray, vec3 aabbMin, vec3 aabbMax)
{
  return intersectAABB(ray.origin, 1.0 / ray.dir, aabbMin, aabbMax);
}

// TODO: implement watertight version of Ray-Triangle intersection, available
// behind a flag

// Implementation of:
// Möller, Tomas; Trumbore, Ben (1997). "Fast, Minimum Storage Ray-Triangle Intersection"
float
intersectTriangle(Ray ray, uint startIndex, uint vertexOffset, inout vec2 uv)
{
  // TODO: pre-process edge?
  // Maybe not really useful if decide to add skinning in shader.
  vec3 v0 = getVertex(vertexOffset, startIndex).position.xyz;
  vec3 v1 = getVertex(vertexOffset, startIndex + 1).position.xyz;
  vec3 v2 = getVertex(vertexOffset, startIndex + 2).position.xyz;

  vec3 e1 = v1 - v0;
  vec3 e2 = v2 - v0;

  vec3 p = cross(ray.dir, e2);
  float det = dot(e1, p);

  // Ray is parralel to edge.
  if (det <= NEG_EPSILON) { return MAX_FLOAT; }
  if (det > NEG_EPSILON && det < EPSILON) { return MAX_FLOAT; }

  float invDet = 1.0 / det;

  // Computes Barycentric coordinates.
  vec3 centered = ray.origin - v0;

  float u = dot(centered, p) * invDet;
  if (u < EPSILON || u > EPSILON1) { return MAX_FLOAT; }

  vec3 q = cross(centered, e1);
  float v = dot(ray.dir, q) * invDet;
  if (v < EPSILON || u + v > EPSILON1) { return MAX_FLOAT; }

  uv = vec2(u, v);
  return dot(e2, q) * invDet;
}

float
sceneHit(Ray ray, inout Intersection intersection)
{
  float dist = MAX_FLOAT;

  // for (uint i = 0; i < lights.length(); ++i)
  // {
  //   Light light = lights[i];
  //   vec3 origin = vec3(light.normal.w, light.tangent.w, light.bitangent.w);
  //   float t = intersectPlane(
  //     ray, - light.normal.xyz, origin, light.tangent.xyz, light.bitangent.xyz
  //   );
  //   if (t > 0.0 && t < dist)
  //   {
  //     intersection.emitter = i;
  //     dist = t;
  //   }
  // }

  for (uint i = 0; i < instances.length(); ++i)
  {
    Instance instance = instances[i];

    // Performs intersection in model space.
    Ray rayModel = transformRay(ray, instance.worldToModel);
    vec3 rayInverseDir = 1.0 / rayModel.dir;

    uint nextIndex = 0;
    while (nextIndex != INVALID_UINT)
    {
      BVHNode node = nodes[instance.bvhRootIndex + nextIndex];

      float d = intersectAABB(rayModel.origin, rayInverseDir, node.min, node.max);
      if (d < MAX_FLOAT && d < dist)
      {
        if (node.primitiveStartIndex != INVALID_UINT)
        {
          vec2 uv = vec2(0.0);
          uint relativeIndex = node.primitiveStartIndex + instance.indexRootIndex;
          float t = intersectTriangle(rayModel, relativeIndex, instance.vertexRootIndex, uv);
          if (t >= 0.0 && t < dist)
          {
            intersection.uv = uv;
            intersection.index = relativeIndex;
            intersection.instance = i;
            intersection.emitter = INVALID_UINT;
            intersection.materialIndex = instance.materialIndex;
            dist = t;
          }
          nextIndex = node.nextNodeIndex;
          continue;
        }
        nextIndex++;
      }
      else
      {
        nextIndex = node.nextNodeIndex;
      }
    }
  }
  return dist;
}

#endif // INTERSECTION_UTILS_H
