// Define bindings and layouts
// Storage buffer for chunk data
@group(0) @binding(0)
var<storage, read> chunkData : array<vec3<i32>>;

// Uniform buffer for matrices (e.g., projection and view)
struct Uniforms {
    projection : mat4x4<f32>,
    view : mat4x4<f32>
};

@group(1) @binding(0)
var<uniform> uniforms : Uniforms;

// Vertex shader inputs and outputs
struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: u32
};

struct VertexOutput {
    @builtin(position) clip: vec4<f32>
};

fn decode_vertex(vertex: u32) -> vec4<f32> {
    // Constants for decoding
    let NUM_BITS_IN_POS: u32 = 10u; // Number of bits per position (matches Rust's logic)
    let CHUNK_SIZE: f32 = 1024.0; // Match the chunk size from Rust code

    // Extract x, y, and z components using bitwise operations
    let x: u32 = (vertex >> (2 * NUM_BITS_IN_POS)) & ((1u << NUM_BITS_IN_POS) - 1);
    let y: u32 = (vertex >> NUM_BITS_IN_POS) & ((1u << NUM_BITS_IN_POS) - 1);
    let z: u32 = vertex & ((1u << NUM_BITS_IN_POS) - 1);

    // Convert to normalized coordinates in the range [0, CHUNK_SIZE]
    let decoded_position: vec4<f32> = vec4<f32>(
        f32(x) / CHUNK_SIZE,
        f32(y) / CHUNK_SIZE,
        f32(z) / CHUNK_SIZE,
        1.0 // Homogeneous coordinate
    );

    return decoded_position;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // read the chunk pos from the storage buffer and make usable
    let chunk_pos = vec4<f32>(f32(chunkData[input.instance_index].x), f32(chunkData[input.instance_index].y), f32(chunkData[input.instance_index].z), 0.0);
    let vertex = decode_vertex(input.position);

    let position = chunk_pos + vertex;

    output.clip = uniforms.projection * uniforms.view * position;

    return output;
}

// Fragment shader inputs and outputs
struct FragmentOutput {
    @location(0) color : vec4<f32>
};

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    // not implemented yet
    var color_palettes: array<vec4<f32>, 8> = array<vec4<f32>, 8> (
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
        vec4<f32>(1.0),
    );

    let selector: u32 = u32(abs(floor(input.clip.y))) % 8u;

    output.color = color_palettes[selector];

    return output;
}
