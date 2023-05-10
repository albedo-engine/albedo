#version 450

#extension GL_GOOGLE_include_directive : enable

#include "structures.glsl"

layout(location=0) in vec4 vPosition;
layout(location=1) in vec4 vNormal;

layout(location=0) out vec3 vPositionWorld;
layout(location=1) out vec3 vNormalWorld;
layout(location=2) out vec2 vUv;

layout (set = 0, binding = 0, std430) readonly buffer InstanceBuffer {
  Instance instances[];
};

void main() {
  Instance instance = instances[gl_InstanceIndex];
  /* Position in World space */
  vPositionWorld = (instance.modelToWorld * vec4(vPosition.xyz, 1.0)).xyz;
  /* Normal in World space */
  vNormalWorld = (instance.modelToWorld * vec4(vNormal.xyz, 1.0)).xyz;
  /* UV */
  vUv = vec2(vPosition.w, vNormal.w);
  vUv = mod(vUv, vec2(1.0, 1.0));
  /* Position in lightmap UV space */
  gl_Position = vec4(vUv * 2.0 - 1.0, 0.0, 1.0);
}
