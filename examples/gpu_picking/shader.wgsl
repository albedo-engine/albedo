struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_index: u32,
};

struct Intersection {
  uv: vec2<f32>,
  index: u32,
  instance: u32,
  materialIndex: u32,
  emitter: u32,
  dist: f32,
  padding_1: f32,
}

struct Uniforms {
    mvpMatrix: mat4x4<f32>,
    color: vec4<f32>,
}

@binding(0) @group(0) var<storage, read> uniforms : array<Uniforms>;
@binding(1) @group(0) var<storage, read> intersection : array<Intersection>;

@vertex
fn vs_main(
    @builtin(instance_index) idx : u32,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.instance_index = idx;
    result.position = uniforms[idx].mvpMatrix * vec4(position.xyz, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec3<f32> = uniforms[vertex.instance_index].color.rgb;
    if (intersection[0].instance == vertex.instance_index) {
        color = vec3<f32>(1.0, 0.0, 0.0);
    }
    return vec4<f32>(color, 1.0);
}
