layout(set = 0, binding = 0, std430) readonly buffer InstanceBuffer {
  Instance instances[];
};

layout(set = 0, binding = 1, std430) readonly buffer BVHNodeBuffer {
  BVHNode nodes[];
};

layout(set = 0, binding = 2, std430) readonly buffer IndexBuffer {
  uint indices[];
};

layout(set = 0, binding = 3, std430) readonly buffer VertexBuffer {
  Vertex vertices[];
};

// @todo: move to uniform?
layout(set = 0, binding = 4, std430) readonly buffer LightBuffer {
  Light lights[];
};

layout(set = 0, binding = 5, std430) readonly buffer MaterialBuffer {
  Material materials[];
};
