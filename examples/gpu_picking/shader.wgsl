struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    projection: mat4x4<f32>,
    mvpMatrix : array<mat4x4<f32>>,
}

@binding(0) @group(0) var<storage, read> uniforms : Uniforms;

@vertex
fn vs_main(
    @builtin(instance_index) idx : u32,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = uniforms.mvpMatrix[idx] * uniforms.projection * position;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {

    var camPos = vec3<f32>(0.0, 0.0, 0.0);
    var lightDir = normalize(vec3(-1.0, -1.0, -1.0));

    // Virtual lighting.

    var cosTheta = dot(lightDir, camPos - vertex.position.xyz);

    return vec4<f32>(vec3<f32>(cosTheta), 1.0);
}
