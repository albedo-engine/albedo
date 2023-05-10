#version 450

#extension GL_GOOGLE_include_directive : enable

#include "structures.glsl"

layout(location=0) in vec4 vPosition;
layout(location=1) in vec4 vNormal;
/*layout(location=2) in vec2 vUv0;
layout(location=2) in vec2 vUv1;*/

layout(location=0) out vec3 vPositionWorld;
layout(location=1) out vec3 vNormalWorld;
layout(location=2) out vec2 vUv;

layout (set = 0, binding = 0, std430) readonly buffer InstanceBuffer {
  Instance instances[];
};

/*
layout (set = 0, binding = 2, std430) readonly buffer IndexBuffer {
  uint indices[];
};

layout (set = 0, binding = 3, std430) readonly buffer VertexBuffer {
  Vertex vertices[];
};
*/

void main() {
  /* Let's hope shaderc will translate that to whatever wgpu SPIRV expects (@builtin(instance_index) in wgsl) */
  Instance instance = instances[gl_InstanceIndex];

  /*
  uint vertexOffset = instance.vertexRootIndex;
  uint index = gl_VertexIndex + instance.indexRootIndex;
  Vertex vertex = getVertex(vertexOffset, index);
  vPosition = vertex.position.xyz;
  vNormal = vertex.normal.xyz;
  vUv0 = vec2(vertex.position.w, vertex.normal.w);
  */

  /* Position in World space */
  vPositionWorld = (instance.modelToWorld * vec4(vPosition.xyz, 1.0)).xyz;
  /* Normal in World space */
  vNormalWorld = (instance.modelToWorld * vec4(vNormal.xyz, 1.0)).xyz;
  /* UV */
  vUv = vec2(vPosition.w, vNormal.w);
  vUv = mod(vUv, vec2(1.0, 1.0));
  //vUv = vUv0;
  /* Position in lightmap UV space */
  gl_Position = vec4(vUv * 2.0 - 1.0, 0.0, 1.0);
  //gl_Position = vec4(vUv1 * 2.0 - 1.0, 0.0, 1.0);
}
