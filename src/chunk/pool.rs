use std::{borrow::Cow, collections::HashMap};

use bytemuck::bytes_of;
use wgpu::{
    util::DrawIndirectArgs, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Operations,
    PipelineLayoutDescriptor, PolygonMode, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp,
};

use crate::{player::Player, util::allocator::Allocator, window_state::WindowState};

use super::{mesher::mesh, traverse, visibility::VisibilityGraph, Chunk, ChunkPos, EncodedVertex};

pub struct ChunkDrawInfo {
    pub vertex_offset: u64,
    pub storage_offset: u64,

    /// offset and length for each face mesh
    pub faces: [(u32, u32); 6],

    /// Used to traverse the world and identify which chunk/chunk faces need to be rendered
    pub vis_graph: VisibilityGraph,
}

/// Manages chunk vertex data. When we want to draw a chunk, we pass a list of
/// chunk positions and faces.
#[derive(Default)]
pub struct ChunkPool {
    vertex_buffer: Option<Buffer>,
    uniform_buffer: Option<Buffer>,
    storage_buffer: Option<Buffer>,
    indirect_buffer: Option<Buffer>,

    vertex_allocator: Allocator,
    storage_allocator: Allocator,

    storage_bind_group: Option<BindGroup>,
    uniform_bind_group: Option<BindGroup>,

    lookup: HashMap<ChunkPos, ChunkDrawInfo>,

    pipeline: Option<RenderPipeline>,
}

impl ChunkPool {
    pub fn initialize(state: &WindowState) -> Self {
        let size = state.device.limits().max_buffer_size;
        let storage_buffer_size = size / 4;

        let desc_vertex = BufferDescriptor {
            label: Some("Chunk pool"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let desc_storage = BufferDescriptor {
            label: Some("Storage pool"),
            size: storage_buffer_size,
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
            size,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let chunk_shader_source =
            std::fs::read_to_string("./assets/shader.wgsl").expect("Chunk shader missing!");
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Chunk shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&chunk_shader_source)),
            });

        let swapchain_format = state.surface.get_capabilities(&state.adapter).formats[0];

        let vertex_buffer = Some(state.device.create_buffer(&desc_vertex));
        let uniform_buffer = Some(state.device.create_buffer(&desc_uniform));
        let storage_buffer = Some(state.device.create_buffer(&desc_storage));
        let indirect_buffer = Some(state.device.create_buffer(&desc_indirect));

        let storage_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Storage layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let storage_bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Storage bind group"),
            layout: &storage_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        });

        let uniform_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Uniform layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let uniform_bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        });

        let render_pipeline = state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Chunk pipeline"),
                layout: Some(
                    &state
                        .device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[
                                &storage_bind_group_layout,
                                &uniform_bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<EncodedVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Uint32,
                        }],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    polygon_mode: PolygonMode::Line,
                    cull_mode: None,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            vertex_allocator: Allocator::new(size),
            storage_allocator: Allocator::new(storage_buffer_size),

            storage_bind_group: Some(storage_bind_group),
            uniform_bind_group: Some(uniform_bind_group),

            vertex_buffer,
            uniform_buffer,
            storage_buffer,
            indirect_buffer,
            lookup: HashMap::new(),
            pipeline: Some(render_pipeline),
        }
    }

    pub fn allocated_percent(&self) -> [f32; 2] {
        [
            self.vertex_allocator.percent_full(),
            self.storage_allocator.percent_full(),
        ]
    }

    /// Upload a chunk so that it can be rendered.
    pub fn add_chunk(&mut self, state: &WindowState, chunk_pos: ChunkPos, chunk: Chunk) {
        log::debug!("ADDING CHUNK {:?}", chunk_pos);
        let vertex_size = std::mem::size_of::<EncodedVertex>() as u32;

        let mesh = mesh(&chunk);
        let mesh_len = vertex_size
            * (mesh
                .iter()
                .fold(0, |acc, item: &Vec<EncodedVertex>| acc + item.len()) as u32);

        let Some(vertex_addr) = self.vertex_allocator.alloc(mesh_len as u64) else {
            return;
        }; // if we can't get a block of memory, just return

        let mut faces = [(0u32, 0u32); 6];
        faces[0].1 = mesh[0].len() as u32;
        for i in 1..6 {
            // offset is the last length plus the last offset
            faces[i].0 = faces[i - 1].1 + faces[i - 1].0;
            faces[i].1 = mesh[i].len() as u32;
        }

        log::debug!("Chunk mesh offset: {}", vertex_addr);
        log::debug!("Chunk mesh: {:?}", mesh);
        log::debug!("Chunk face data: {:?}", faces);

        // upload vertex data
        let data: Vec<_> = mesh.into_iter().flatten().map(|x| x.to_untyped()).collect();
        state.queue.write_buffer(
            self.vertex_buffer
                .as_ref()
                .expect("No vertex buffer found! It should be here."),
            vertex_addr,
            bytemuck::cast_slice(data.as_slice()),
        );

        // We include an additional 0 so that we don't have to do any trickery trying to get the alignment correct
        let pos = [32 * chunk_pos.0, 32 * chunk_pos.1, 32 * chunk_pos.2, 0];
        let pos_length = std::mem::size_of::<[i32; 4]>();

        let Some(storage_addr) = self.storage_allocator.alloc(pos_length as u64) else {
            return;
        }; // if we can't get a block of memory, just return
           // upload storage data

        log::debug!("Storage translation offset: {}", storage_addr);
        log::debug!(
            "Inserting into storage buffer: {:?} (size={})",
            pos,
            pos_length
        );

        state.queue.write_buffer(
            self.storage_buffer
                .as_ref()
                .expect("No storage buffer found! It should be here."),
            storage_addr,
            bytemuck::bytes_of(&pos),
        );

        let vis_graph = VisibilityGraph::from_chunk(&chunk);

        // create the chunk info so that we can create indirect draw calls
        // from this
        self.lookup.insert(
            chunk_pos,
            ChunkDrawInfo {
                // these offsets are the indices into the buffer, not the actual memory location!
                vertex_offset: vertex_addr / vertex_size as u64,
                storage_offset: storage_addr / pos_length as u64,
                faces,
                vis_graph,
            },
        );

        log::debug!("DONE UPLOADING CHUNK");
    }

    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        let Some(chunk_info) = self.lookup.remove(&pos) else {
            return;
        };

        self.vertex_allocator.dealloc(chunk_info.vertex_offset);
        self.storage_allocator.dealloc(chunk_info.storage_offset);
    }

    pub fn render(&self, state: &WindowState, player: &Player, _build_list: ()) {
        // Build a vec of all the chunk faces that need to be drawn, then upload it to the GPU
        let draw_list = traverse::build_draw_list(&self.lookup, player);
        let call_count = draw_list.len() as u32;
        self.upload_draw_buffer(state, draw_list);

        // Upload player projection and view matrices
        self.upload_player_uniforms(state, player);

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
            render_pass.push_debug_group("Prepare data for draw.");
            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));

            render_pass.set_bind_group(0, self.storage_bind_group.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(1, self.uniform_bind_group.as_ref().unwrap(), &[]);

            render_pass.pop_debug_group();
            render_pass.insert_debug_marker("Draw!");
            render_pass.multi_draw_indirect(self.indirect_buffer.as_ref().unwrap(), 0, call_count);
        }
        state.queue.submit([encoder.finish()]);
        frame.present();
    }

    fn upload_player_uniforms(&self, state: &WindowState, player: &Player) {
        let Some(buf) = self.uniform_buffer.as_ref() else {
            return;
        };

        let p: [[f32; 4]; 4] = player.get_projection().into();
        let x = bytes_of(&p);

        let v: [[f32; 4]; 4] = player.get_view().into();
        let y = bytes_of(&v);

        state.queue.write_buffer(buf, 0, &[x, y].concat());
    }

    fn upload_draw_buffer(&self, state: &WindowState, indirect_data: Vec<DrawIndirectArgs>) {
        // Encode the structs as bytes so that they can be uploaded
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
    }
}
