use zero::{impl_simple_buffer, prelude::wgpu::*, prelude::*};

use crate::physics::{Collider, Collision, Rectangle};

pub struct Crate {
    transform: Transform,
    disabled: bool,
}

impl Crate {
    pub fn new(position: Vector3<f32>) -> Self {
        Self {
            disabled: false,
            transform: Transform {
                translation: position,
                ..Default::default()
            },
        }
    }

    #[inline]
    pub fn rect(&self, rect_width: f32, rect_height: f32) -> Rectangle {
        Rectangle::from_center(
            self.transform.translation.truncate(),
            rect_width,
            rect_height,
        )
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CrateUniform {
    transform: [[f32; 4]; 4],
    disabled: u32,
}

impl From<&Crate> for CrateUniform {
    fn from(value: &Crate) -> Self {
        CrateUniform {
            transform: Matrix4::<f32>::from(&value.transform).into(),
            disabled: value.disabled.into(),
        }
    }
}

const MAX_CRATES: usize = 64;
pub struct Crates {
    crates: Vec<Crate>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CratesUniform {
    crates: [CrateUniform; MAX_CRATES],
}

impl From<&Crates> for CratesUniform {
    fn from(value: &Crates) -> Self {
        let mut crates = [CrateUniform::default(); MAX_CRATES];
        for (i, c) in value.crates.iter().enumerate() {
            crates[i] = c.into();
        }
        Self { crates }
    }
}

impl_simple_buffer!(
    Crates,
    CratesUniform,
    CratesResources,
    CratesHandle,
    CratesBindGroup,
    { BufferUsages::VERTEX | BufferUsages::COPY_DST },
    { ShaderStages::VERTEX | ShaderStages::FRAGMENT },
    { BufferBindingType::Storage { read_only: true } }
);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CrateInstanceVertex {
    pub transform_0: [f32; 4],
    pub transform_1: [f32; 4],
    pub transform_2: [f32; 4],
    pub transform_3: [f32; 4],
    pub disabled: i32,
}

impl VertexLayout for CrateInstanceVertex {
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
                    format: VertexFormat::Sint32,
                },
            ],
        }
    }
}

pub struct CratesRenderCommand {
    pub pipeline_id: ResourceId,
    pub mesh_id: ResourceId,
    pub instance_buffer_id: ResourceId,
    pub bind_groups: [ResourceId; 2],
    pub num: u32,
}

impl RenderCommand for CratesRenderCommand {
    fn execute<'a>(&self, render_pass: &mut RenderPass<'a>, storage: &'a CurrentFrameStorage) {
        render_pass.set_pipeline(storage.get_pipeline(self.pipeline_id));
        for (i, bg) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, storage.get_bind_group(*bg), &[]);
        }

        let mesh = storage.get_mesh(self.mesh_id);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        let instance_buffer = storage.get_buffer(self.instance_buffer_id);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        let index_buffer = mesh.index_buffer.as_ref().unwrap();
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.num);
    }
}

pub struct CratePack {
    pub mesh_id: ResourceId,

    pub material: crate::ColorMaterial,
    pub material_handle: crate::ColorMaterialHandle,
    pub material_bind_group: crate::ColorMaterialBindGroup,

    pub crates: Crates,
    pub crates_handle: CratesHandle,
    pub rect_width: f32,
    pub rect_height: f32,
}

impl CratePack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        center: Vector3<f32>,
        rows: u32,
        cols: u32,
        width: f32,
        height: f32,
        gap_x: f32,
        gap_y: f32,
        color: [f32; 4],
    ) -> Self {
        let bottom_left = center
            - Vector3::new(
                (gap_x + width) / 2.0 * (cols - 1) as f32,
                (gap_y + height) / 2.0 * (rows - 1) as f32,
                0.0,
            );
        let mut crates = vec![];
        for x in 0..cols {
            for y in 0..rows {
                let c = Crate::new(Vector3::new(
                    bottom_left.x + x as f32 * (width + gap_x),
                    bottom_left.y + y as f32 * (height + gap_y),
                    0.0,
                ));
                crates.push(c);
            }
        }
        let crates = Crates { crates };

        let mesh: Mesh = Quad::new(width, height).into();
        let mesh_id = storage.insert_mesh(mesh.build(renderer));

        let material = crate::ColorMaterial { color };
        let material_handle = crate::ColorMaterialHandle::new(storage, material.build(renderer));
        let material_bind_group =
            crate::ColorMaterialBindGroup::new(renderer, storage, &material_handle);

        let crates_handle = CratesHandle::new(storage, crates.build(renderer));

        Self {
            mesh_id,
            material,
            material_handle,
            material_bind_group,
            crates,
            crates_handle,
            rect_width: width,
            rect_height: height,
        }
    }

    pub fn render_sync(&self, renderer: &Renderer, storage: &RenderStorage) {
        self.crates_handle.update(renderer, storage, &self.crates)
    }

    #[inline]
    pub fn render_commands(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> CratesRenderCommand {
        CratesRenderCommand {
            pipeline_id,
            mesh_id: self.mesh_id,
            instance_buffer_id: self.crates_handle.buffer_id,
            bind_groups: [self.material_bind_group.0, camera_bind_group],
            num: self.crates.crates.len() as u32,
        }
    }
}

impl Collider for CratePack {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        None
    }

    #[inline]
    fn collides_mut(&mut self, other: &impl Collider) -> Option<Collision> {
        for c in self.crates.crates.iter_mut() {
            if !c.disabled {
                let crate_rect = c.rect(self.rect_width, self.rect_height);
                if let Some(collision) = crate_rect.collides(other) {
                    c.disabled = true;
                    return Some(collision);
                }
            }
        }
        None
    }
}
