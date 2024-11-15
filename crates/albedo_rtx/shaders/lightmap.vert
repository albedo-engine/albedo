#version 450

#include "imports/structures.glsl"

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
  vNormalWorld = (instance.modelToWorld * vec4(vNormal.xyz, 0.0)).xyz;
  /* UV */
  vUv = vec2(vPosition.w, vNormal.w);
  vUv *= 0.99999;
  vUv = mod(vUv, vec2(1.0, 1.0));

  bool flipY = true;
  if (flipY) {
    vUv.y = 1.0 - vUv.y;
  }

  /* Position in lightmap UV space */
  gl_Position = vec4(vUv * 2.0 - 1.0, 0.0, 1.0);
}
