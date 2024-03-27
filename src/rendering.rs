use zero::{impl_simple_sized_gpu_buffer, prelude::*};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceVertex {
    pub transform_0: [f32; 4],
    pub transform_1: [f32; 4],
    pub transform_2: [f32; 4],
    pub transform_3: [f32; 4],
    pub color: [f32; 4],
    pub disabled: i32,
}

impl VertexLayout for InstanceVertex {
    fn layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 20]>() as BufferAddress,
                    shader_location: 10,
                    format: VertexFormat::Sint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceUniform {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub disabled: u32,
}

impl_simple_sized_gpu_buffer!(InstancesBuffer, InstancesBufferResources, {
    BufferUsages::VERTEX | BufferUsages::COPY_DST
});

pub struct InstanceBufferHandle {
    buffer_id: ResourceId,
}

impl InstanceBufferHandle {
    pub fn new(storage: &mut RenderStorage, resource: InstancesBufferResources) -> Self {
        Self {
            buffer_id: storage.insert_buffer(resource.buffer),
        }
    }

    pub fn update(
        &self,
        renderer: &Renderer,
        storage: &RenderStorage,
        offset: BufferAddress,
        data: &[impl bytemuck::NoUninit],
    ) {
        renderer.queue().write_buffer(
            storage.get_buffer(self.buffer_id),
            offset,
            bytemuck::cast_slice(data),
        );
    }
}

pub struct Instances {
    pub mesh_id: ResourceId,
    pub instance_buffer_handle: InstanceBufferHandle,
    pub instance_num: u32,
}

impl Instances {
    pub fn new<M: Into<Mesh>>(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        mesh: M,
        num: u32,
    ) -> Self {
        let mesh: Mesh = mesh.into();
        let mesh_id = storage.insert_mesh(mesh.build(renderer));

        let instance_buffer = InstancesBuffer {
            size: num as u64 * std::mem::size_of::<InstanceUniform>() as u64,
        };
        let instance_buffer_resource = instance_buffer.build(renderer);
        let instance_buffer_handle = InstanceBufferHandle::new(storage, instance_buffer_resource);
        Self {
            mesh_id,
            instance_buffer_handle,
            instance_num: num,
        }
    }

    pub fn render_command(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> InstancesRenderCommand {
        InstancesRenderCommand {
            pipeline_id,
            mesh_id: self.mesh_id,
            instance_buffer_id: self.instance_buffer_handle.buffer_id,
            camera_bind_group,
            instance_num: self.instance_num,
        }
    }
}

pub struct InstancesRenderCommand {
    pub pipeline_id: ResourceId,
    pub mesh_id: ResourceId,
    pub instance_buffer_id: ResourceId,
    pub camera_bind_group: ResourceId,
    pub instance_num: u32,
}

impl RenderCommand for InstancesRenderCommand {
    fn execute<'a>(&self, render_pass: &mut RenderPass<'a>, storage: &'a CurrentFrameStorage) {
        render_pass.set_pipeline(storage.get_pipeline(self.pipeline_id));
        render_pass.set_bind_group(0, storage.get_bind_group(self.camera_bind_group), &[]);

        let mesh = storage.get_mesh(self.mesh_id);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        let instance_buffer = storage.get_buffer(self.instance_buffer_id);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        let index_buffer = mesh.index_buffer.as_ref().unwrap();
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instance_num);
    }
}
