// Define bindings and layouts
// Storage buffer for chunk data
@group(0) @binding(0)
var<storage, read> chunkData : array<vec4<i32>>;

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

fn create_translation_matrix(translation: vec4<i32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),  // First column
        vec4<f32>(0.0, 1.0, 0.0, 0.0),  // Second column
        vec4<f32>(0.0, 0.0, 1.0, 0.0),  // Third column
        vec4<f32>(f32(translation.x), f32(translation.y), f32(translation.z), 1.0) // Fourth column
    );
}

fn decode_vertex(vertex: u32) -> vec4<f32> {
    
    let NUM_BITS_IN_POS: u32 = 6u;
    let offset = 63u;

    let t1 = offset << NUM_BITS_IN_POS * 0;
    let z: u32 = (vertex & t1) >> (NUM_BITS_IN_POS * 0);
    
    let t2 = offset << NUM_BITS_IN_POS * 1;
    let y: u32 = (vertex & t2) >> (NUM_BITS_IN_POS * 1);
    
    let t3 = offset << NUM_BITS_IN_POS * 2;
    let x: u32 = (vertex & t3) >> (NUM_BITS_IN_POS * 2);

    let decoded_position: vec4<f32> = vec4<f32>(
        f32(x),
        f32(y),
        f32(z),
        1.0
    );

    return decoded_position;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // read the chunk pos from the storage buffer and make usable
    let vertex = decode_vertex(input.position);
    let model = create_translation_matrix(chunkData[input.instance_index]);

    output.clip = uniforms.projection * uniforms.view * model * vertex;

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
