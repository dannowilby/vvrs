#![allow(dead_code)]

use std::{borrow::Cow, collections::HashMap};

use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Operations,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, StoreOp,
};

use crate::{player::Player, window_state::WindowState};

struct ChunkDrawInfo {
    offset: u32, // offset in the vertex buffer
}

/// Manages chunk vertex data.
#[derive(Default)]
pub struct ChunkPool {
    vertex_buffer: Option<Buffer>,
    indirect_buffer: Option<Buffer>,
    lookup: HashMap<(i32, i32, i32), ChunkDrawInfo>, // maps chunk positions into the buffer memory offset
    free: Vec<u32>,                                  // keeps track of where any new buffers can go
    pipeline: Option<RenderPipeline>,
}

impl ChunkPool {
    pub fn initialize(state: &WindowState) -> Self {
        let size = state.device.limits().max_buffer_size;
        let usage = BufferUsages::VERTEX | BufferUsages::COPY_DST;
        let desc_vertex = BufferDescriptor {
            label: Some("Chunk pool"),
            size,
            usage,
            mapped_at_creation: false,
        };

        let desc_indirect = BufferDescriptor {
            label: Some("Indirect pool"),
            size,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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

        Self {
            vertex_buffer: Some(state.device.create_buffer(&desc_vertex)),
            indirect_buffer: Some(state.device.create_buffer(&desc_indirect)),
            lookup: HashMap::new(),
            free: Vec::new(),
            pipeline: Some(render_pipeline),
        }
    }

    /// Recalculates the chunks that need to be loaded, and loads them.
    pub fn update_chunks(&self, _player: &Player) {
        // queue: &Queue, _chunk: &Chunk) {

        println!("updating");

        // let chunk_mesh = mesh();
        // queue.write_buffer(&self.buffer, 0);
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
