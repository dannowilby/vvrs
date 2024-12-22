/// TODO:
/// - Create bind group layout for uniform and storage buffers
/// - upload projection/view matrices to uniform buffer
/// - set the buffers when drawing
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use wgpu::{
    util::DrawIndirectArgs, Buffer, BufferDescriptor, BufferUsages, Color,
    CommandEncoderDescriptor, Operations, PipelineLayoutDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp,
};

use crate::{chunk::MAX_CHUNK_MEMORY_USAGE, player::Player, window_state::WindowState};

use super::{mesher::mesh, Chunk, ChunkPos};

struct ChunkDrawInfo {
    pub offset: u64,
    /// offset and length for each face mesh
    pub faces: [(u32, u32); 6],
}

/// Manages chunk vertex data.
#[allow(dead_code)]
#[derive(Default)]
pub struct ChunkPool {
    vertex_buffer: Option<Buffer>,
    uniform_buffer: Option<Buffer>,
    storage_buffer: Option<Buffer>,
    indirect_buffer: Option<Buffer>,

    num_meshes: u64,

    /// Maps chunk position/id into its render info,
    /// we also use this to check which chunks are loaded or not.
    lookup: HashMap<(i32, i32, i32), ChunkDrawInfo>,

    free: Vec<u64>, // keeps track of where any new buffers can go
    pipeline: Option<RenderPipeline>,
}

impl ChunkPool {
    #[allow(unused_variables, unreachable_code)]
    pub fn initialize(state: &WindowState) -> Self {
        let size = state.device.limits().max_buffer_size;
        let num_meshes = size / MAX_CHUNK_MEMORY_USAGE as u64;

        let desc_vertex = BufferDescriptor {
            label: Some("Chunk pool"),
            size: num_meshes * MAX_CHUNK_MEMORY_USAGE as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let desc_storage = BufferDescriptor {
            label: Some("Storage pool"),
            size: num_meshes * std::mem::size_of::<ChunkPos>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let desc_uniform = BufferDescriptor {
            label: Some("Uniform pool"),
            size: 2 * std::mem::size_of::<cgmath::Matrix4<f32>>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let desc_indirect = BufferDescriptor {
            label: Some("Indirect pool"),
            size: num_meshes * std::mem::size_of::<DrawIndirectArgs>() as u64,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let chunk_shader_source =
            std::fs::read_to_string("./assets/shader.wgsl").expect("Chunk shader missing!"); // change to proper logging at some point
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Chunk shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&chunk_shader_source)),
            });

        let swapchain_format = state.surface.get_capabilities(&state.adapter).formats[0];

        let render_pipeline = state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Chunk pipeline"),
                layout: Some(
                    &state
                        .device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[], // <------------------------------ add the uniform and storage buffer layouts here
                            push_constant_ranges: &[],
                        }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[], // <-------------------------------------------- `Vertex::desc()` goes here
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(), // <--------------------------------------- May want to edit this to change defaults
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let free = (0..(size / MAX_CHUNK_MEMORY_USAGE as u64)).collect();

        Self {
            num_meshes,
            vertex_buffer: Some(state.device.create_buffer(&desc_vertex)),
            uniform_buffer: Some(state.device.create_buffer(&desc_uniform)),
            storage_buffer: Some(state.device.create_buffer(&desc_storage)),
            indirect_buffer: Some(state.device.create_buffer(&desc_indirect)),
            lookup: HashMap::new(),
            free,
            pipeline: Some(render_pipeline),
        }
    }

    pub fn upload_chunk(&mut self, state: &WindowState, chunk_pos: (i32, i32, i32)) {
        // somehow get the chunk, either from memory or by generating it
        let chunk = Chunk::random();

        let mesh = mesh(&chunk);
        let addr = self.free.pop().expect("Vertex buffer is full!");

        let mut faces_offsets = [0u32; 6];

        for i in 1..6 {
            faces_offsets[i] = mesh[i - 1].len() as u32 + faces_offsets[i - 1];
        }

        // create the chunk info so that we can create indirect draw calls
        // from this
        self.lookup.insert(
            chunk_pos,
            ChunkDrawInfo {
                offset: addr,
                faces: [
                    (faces_offsets[0], mesh[0].len() as u32),
                    (faces_offsets[1], mesh[1].len() as u32),
                    (faces_offsets[2], mesh[2].len() as u32),
                    (faces_offsets[3], mesh[3].len() as u32),
                    (faces_offsets[4], mesh[4].len() as u32),
                    (faces_offsets[5], mesh[5].len() as u32),
                ],
            },
        );

        let vertex_offset = addr * MAX_CHUNK_MEMORY_USAGE as u64;
        let storage_offset = addr * std::mem::size_of::<ChunkPos>() as u64;

        // upload vertex data
        let data: Vec<_> = mesh.into_iter().flatten().map(|x| x.to_untyped()).collect();
        state.queue.write_buffer(
            self.vertex_buffer
                .as_ref()
                .expect("No vertex buffer found! It should be here."),
            vertex_offset,
            bytemuck::cast_slice(data.as_slice()),
        );

        // upload storage data
        let data: (i32, i32, i32) = chunk_pos;
        state.queue.write_buffer(
            self.storage_buffer
                .as_ref()
                .expect("No uniform buffer found! It should be here."),
            storage_offset,
            bytemuck::bytes_of(&[data.0, data.1, data.2]),
        );
    }

    /// Recalculates the chunks that need to be loaded, and loads them.
    pub fn update_chunks(&mut self, state: &WindowState, player: &Player) {
        let mut chunks_to_remove: HashSet<(i32, i32, i32)> = self.lookup.keys().cloned().collect();
        let mut chunks_to_add = Vec::<(i32, i32, i32)>::new();

        let pos = player.get_chunk_pos();
        let r = player.load_radius as i32;

        for x in (pos.0 - r)..=(pos.0 + r) {
            for y in (pos.0 - r)..=(pos.0 + r) {
                for z in (pos.0 - r)..=(pos.0 + r) {
                    let new_pos = (x, y, z);

                    if !self.lookup.contains_key(&new_pos) {
                        chunks_to_add.push(new_pos);
                    }

                    chunks_to_remove.remove(&new_pos);
                }
            }
        }

        // remove the chunks and add their memory address to the free list
        for chunk_pos in chunks_to_remove {
            let Some(chunk_info) = self.lookup.get(&chunk_pos) else {
                continue;
            };

            self.free.push(chunk_info.offset);
            self.lookup.remove(&chunk_pos);
        }

        for chunk_pos in chunks_to_add {
            self.upload_chunk(state, chunk_pos);
        }
    }

    pub fn render(&self, state: &WindowState, player: &Player) {
        let call_count = self.build_draw_list(state, player);

        let frame = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Chunk renderer"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Chunk pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));

            // <------------------------------------ set bind groups here

            render_pass.multi_draw_indirect(self.indirect_buffer.as_ref().unwrap(), 0, call_count);
        }
        state.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn build_draw_list(&self, state: &WindowState, _player: &Player) -> u32 {
        let mut indirect_data = vec![];

        self.lookup.values().for_each(|x| {
            let addr = x.offset;
            let vertex_offset = addr * MAX_CHUNK_MEMORY_USAGE as u64;
            let storage_offset = addr * std::mem::size_of::<ChunkPos>() as u64;

            let face_offset = x.faces[0].0;
            let face_count = x.faces[0].1;
            indirect_data.push(DrawIndirectArgs {
                vertex_count: face_count,
                instance_count: 1,
                first_vertex: vertex_offset as u32 + face_offset,
                first_instance: storage_offset as u32, // use first instance to index into the uniform buffer
            });
        });
        let call_count = indirect_data.len() as u32;

        // submit the data
        let data = indirect_data
            .iter()
            .flat_map(|f| f.as_bytes())
            .cloned()
            .collect::<Vec<_>>();
        state.queue.write_buffer(
            self.indirect_buffer
                .as_ref()
                .expect("Should be an indirect buffer here!"),
            0,
            data.as_slice(),
        );

        call_count
    }
}
