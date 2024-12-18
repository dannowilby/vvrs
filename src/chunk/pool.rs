use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use super::Chunk;


// we should have some way of mapping a chunk's position into a range of memory
// in the buffer. This way writing new chunks should not be so bad. Also, using
// the max_buffer_size, we can calculate how many meshes we can fit into the
// buffer, so we should also do that.
pub struct ChunkPool {
    buffer: Buffer,
}

impl ChunkPool {

    pub fn initialize(device: &Device) -> Self {

        let size = device.limits().max_buffer_size;
        let usage = BufferUsages::VERTEX | BufferUsages::COPY_DST;
        let desc = BufferDescriptor { label: Some("Chunk pool"), size, usage, mapped_at_creation: false };

        Self { buffer: device.create_buffer(&desc) }
    }

    // this function and the routine to mesh/calculate new and old chunks are
    // tightly coupled, so it might be better to think through in a new way.
    pub fn write_chunk(&self, queue: &Queue, _chunk: &Chunk) {

        queue.write_buffer(&self.buffer, 0, &[0, 0, 1]);
    }

}