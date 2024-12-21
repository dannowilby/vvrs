#![allow(dead_code)]

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Operations,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, StoreOp,
};

use crate::{chunk::MAX_CHUNK_MEMORY_USAGE, player::Player, window_state::WindowState};

use super::{
    mesher::mesh,
    Chunk
};

/// Still need to set up uniform buffer, indirect call creation, and properly
/// format render pipeline.
struct ChunkDrawInfo {
    pub vertex_offset: u32, // offset in the vertex buffer
    pub uniform_offset: u32,
    /// offset and length for each face mesh
    pub faces: [(u32, u32); 6],
}

/// Manages chunk vertex data.
#[derive(Default)]
pub struct ChunkPool {
    vertex_buffer: Option<Buffer>,
    indirect_buffer: Option<Buffer>,

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
        let desc_vertex = BufferDescriptor {
            label: Some("Chunk pool"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        todo!("Add uniform buffer for chunk translation matrices");
        let _desc_uniform = BufferDescriptor {
            label: Some("Uniform pool"),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let desc_indirect = BufferDescriptor {
            label: Some("Indirect pool"),
            size,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let chunk_shader_source =
            std::fs::read_to_string("./assets/shader.wgsl").expect("Chunk shader missing!"); // change to proper logging at some point
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
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
                            bind_group_layouts: &[],
                            push_constant_ranges: &[],
                        }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let free = (0..(size / MAX_CHUNK_MEMORY_USAGE as u64)).collect();

        Self {
            vertex_buffer: Some(state.device.create_buffer(&desc_vertex)),
            indirect_buffer: Some(state.device.create_buffer(&desc_indirect)),
            lookup: HashMap::new(),
            free,
            pipeline: Some(render_pipeline),
        }
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

            self.free.push(chunk_info.vertex_offset as u64);
            self.lookup.remove(&chunk_pos);
        }

        for chunk_pos in chunks_to_add {
            // somehow get the chunk, either from memory or by generating it
            let chunk = Chunk::default();

            let mesh = mesh(&chunk);
            let addr = self.free.pop().expect("Vertex buffer is full!") as u32;

            let mut faces_offsets = [0u32; 6];

            for i in 1..6 {
                faces_offsets[i] = mesh[i - 1].len() as u32 + faces_offsets[i - 1];
            }

            // create the chunk info so that we can create indirect draw calls
            // from this
            self.lookup.insert(
                chunk_pos,
                ChunkDrawInfo {
                    vertex_offset: addr,
                    uniform_offset: 0,
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

            let data: Vec<_> = mesh.into_iter().flatten().map(|x| x.to_untyped()).collect();

            state.queue.write_buffer(
                self.vertex_buffer
                    .as_ref()
                    .expect("No vertex buffer found! It should be here."),
                addr as u64,
                bytemuck::cast_slice(data.as_slice()),
            );
        }
    }

    pub fn render(&self, state: &WindowState, _player: &Player) {
        // create render list using player camera most likely

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
            render_pass.multi_draw_indirect(self.indirect_buffer.as_ref().unwrap(), 0, 0);
        }
        state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
