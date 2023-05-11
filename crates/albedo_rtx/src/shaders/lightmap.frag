#version 450

#include "utils/math.glsl"
#include "structures.glsl"

#define MAX_SAMPLES 16

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

layout (set = 0, binding = 2, std430) readonly buffer IndexBuffer {
  uint indices[];
};

layout (set = 0, binding = 3, std430) readonly buffer VertexBuffer {
  Vertex vertices[];
};

layout (set = 0, binding = 4) uniform GlobalUniformBuffer {
  GlobalUniforms global;
};

#include "utils/intersection_utils.glsl"
#include "utils/sampling.glsl"

void main() {
  uint randState = uint(
    gl_FragCoord.x * uint(1973)
    + gl_FragCoord.y * uint(9277)
    + uint(global.seed) * uint(26699)
  ) | uint(1);

  float accumulate = 0.0;
  for(int i = 0; i < MAX_SAMPLES; ++i) {
    vec3 normal = vNormalWorld;
    vec3 worldUp = abs(normal.z) < 0.9999 ? vec3(0, 0, 1) : vec3(1, 0, 0);
    vec3 tangent = normalize(cross(worldUp, normal));
    vec3 bitangent = cross(normal, tangent);
    vec3 localDir = randomCosineWeightedVector(randState);
    vec3 rayDir = normalize(project(localDir, normal, tangent, bitangent));

    Ray ray;
    ray.origin = vPositionWorld;
    ray.dir = rayDir;

    Intersection intersection;
    intersection.index = INVALID_UINT;
    intersection.instance = INVALID_UINT;
    intersection.emitter = INVALID_UINT;
    intersection.dist = sceneHit(ray, intersection);
    if(intersection.dist > 1.0)
      accumulate += 1.0;
  }
  accumulate /= MAX_SAMPLES;

  // outColor = vec4(vec3(1.0 - accumulate), 1.0);
  outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
