#version 450

#include "imports/math.glsl"
#include "imports/structures.glsl"

#define MAX_SAMPLES 128

layout(location=0) in vec3 vPositionWorld;
layout(location=1) in vec3 vNormalWorld;
layout(location=2) in vec2 vUv;

layout(location = 0) out vec4 outColor;

layout (set = 0, binding = 0, std430) readonly buffer InstanceBuffer {
  Instance instances[];
};

layout (set = 0, binding = 1, std430) readonly buffer BVHNodeBuffer {
  BVHNode nodes[];
};

layout (set = 0, binding = 2, std430) readonly buffer CWBVHTriangleBuffer {
  vec4 trianglesCWBVH[];
};

layout (set = 0, binding = 3, std430) readonly buffer VertexBuffer {
  Vertex vertices[];
};

layout (set = 0, binding = 4) uniform GlobalUniformBuffer {
  GlobalUniforms global;
};

#include "imports/common.glsl"
#include "imports/intersection_utils.glsl"
#include "imports/sampling.glsl"

void main() {
  uint randState = uint(
    gl_FragCoord.x * uint(1973)
    + gl_FragCoord.y * uint(9277)
    + uint(global.seed) * uint(26699)
  ) | uint(1);

  vec3 normal = normalize(vNormalWorld);
  float radius = 5.0;

  float accumulate = 0.0;
  for(int i = 0; i < MAX_SAMPLES; ++i) {
    vec3 worldUp = abs(normal.z) < 0.9999 ? vec3(0, 0, 1) : vec3(1, 0, 0);
    vec3 tangent = normalize(cross(worldUp, normal));
    vec3 bitangent = cross(normal, tangent);
    vec3 localDir = randomCosineWeightedVector(randState);
    vec3 rayDir = normalize(project(localDir, normal, tangent, bitangent));

    Ray ray;
    ray.origin = vPositionWorld;
    ray.dir = rayDir;

    Intersection intersection = sceneHit(ray);
    if(intersection.dist >= radius)
      accumulate += 1.0;
  }
  accumulate /= float(MAX_SAMPLES);

  outColor = vec4(vec3(accumulate), 1.0);
}
