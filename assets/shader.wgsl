
@group(0) @binding(0)
var<storage, read> instance_attributes: array<vec3<i32>>;

@group(1) @binding(0)
var<uniform> projection: mat4x4<f32>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: u32
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>
}

fn unpack_vertex(pos: u32) -> vec4<f32> {

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {

    let pos = unpack_vertex(input.position);
    let model = instance_attributes[input.instance_index];
    let output = VertexOutput(projection * pos);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}