use winit::{dpi::PhysicalSize, event::ElementState, keyboard::Key, window::Window};
use zero::{const_vec, impl_simple_buffer, impl_simple_sized_gpu_buffer, prelude::*};

use crate::{
    ball::Ball, border::Border, crates::CratePack, physics::Rectangle, platform::Platform,
};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorMaterialUniform {
    color: [f32; 4],
}

impl From<&ColorMaterial> for ColorMaterialUniform {
    fn from(value: &ColorMaterial) -> Self {
        Self { color: value.color }
    }
}

#[derive(Debug)]
pub struct ColorMaterial {
    pub color: [f32; 4],
}

impl_simple_buffer!(
    ColorMaterial,
    ColorMaterialUniform,
    ColorMaterialResources,
    ColorMaterialHandle,
    ColorMaterialBindGroup,
    { BufferUsages::UNIFORM | BufferUsages::COPY_DST },
    { ShaderStages::FRAGMENT },
    { BufferBindingType::Uniform }
);

pub struct GameCamera {
    camera: Camera,
    handle: CameraHandle,
    bind_group: CameraBindGroup,
}

impl GameCamera {
    pub fn new(renderer: &Renderer, storage: &mut RenderStorage, position: [f32; 3]) -> Self {
        let camera = Camera::Orthogonal(OrthogonalCamera {
            position: position.into(),
            direction: -Vector3::unit_z(),
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near: 0.1,
            far: 100.0,
        });
        let handle = CameraHandle::new(storage, camera.build(renderer));
        let bind_group = CameraBindGroup::new(renderer, storage, &handle);

        Self {
            camera,
            handle,
            bind_group,
        }
    }
}

pub struct GameObject {
    pub mesh_id: ResourceId,

    pub material: ColorMaterial,
    pub material_handle: ColorMaterialHandle,
    pub material_bind_group: ColorMaterialBindGroup,

    pub transform: Transform,
    pub transform_handle: TransformHandle,
    pub transform_bind_group: TransformBindGroup,

    pub rect_width: f32,
    pub rect_height: f32,
}

impl GameObject {
    pub fn new<M: Into<Mesh>>(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        mesh: M,
        rect_width: f32,
        rect_height: f32,
        color: [f32; 4],
        position: Vector3<f32>,
    ) -> Self {
        let mesh: Mesh = mesh.into();
        let mesh_id = storage.insert_mesh(mesh.build(renderer));

        let material = ColorMaterial { color };
        let material_handle = ColorMaterialHandle::new(storage, material.build(renderer));
        let material_bind_group = ColorMaterialBindGroup::new(renderer, storage, &material_handle);

        let transform = Transform {
            translation: position,
            ..Default::default()
        };
        let transform_handle = TransformHandle::new(storage, transform.build(renderer));
        let transform_bind_group = TransformBindGroup::new(renderer, storage, &transform_handle);

        Self {
            mesh_id,
            material,
            material_handle,
            material_bind_group,
            transform,
            transform_handle,
            transform_bind_group,
            rect_width,
            rect_height,
        }
    }

    pub fn command(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> MeshRenderCommand {
        MeshRenderCommand {
            pipeline_id,
            mesh_id: self.mesh_id,
            index_slice: None,
            vertex_slice: None,
            scissor_rect: None,
            bind_groups: const_vec![
                self.material_bind_group.0,
                self.transform_bind_group.0,
                camera_bind_group,
            ],
        }
    }

    pub fn rect(&self) -> Rectangle {
        Rectangle::from_center(
            self.transform.translation.truncate(),
            self.rect_width,
            self.rect_height,
        )
    }

    pub fn update_transform(&self, renderer: &Renderer, storage: &RenderStorage) {
        self.transform_handle
            .update(renderer, storage, &self.transform);
    }
}

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
    pub box_instance_buffer_handle: InstanceBufferHandle,
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
            box_instance_buffer_handle: instance_buffer_handle,
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
            instance_buffer_id: self.box_instance_buffer_handle.buffer_id,
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

pub struct Game {
    renderer: Renderer,
    storage: RenderStorage,

    color_pipeline_id: ResourceId,
    instance_pipeline_id: ResourceId,
    phase: RenderPhase,

    camera: GameCamera,

    box_instances: Instances,

    border: Border,
    ball: Ball,
    platform: Platform,
    crate_pack: CratePack,
}

impl Game {
    pub fn new(window: &Window) -> Self {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut storage = RenderStorage::default();

        storage.register_bind_group_layout::<CameraBindGroup>(&renderer);
        storage.register_bind_group_layout::<ColorMaterialBindGroup>(&renderer);
        storage.register_bind_group_layout::<TransformBindGroup>(&renderer);

        let color_pipeline = PipelineBuilder {
            shader_path: "./shaders/color.wgsl",
            label: Some("color_pipeline"),
            layout_descriptor: Some(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    storage.get_bind_group_layout::<ColorMaterialBindGroup>(),
                    storage.get_bind_group_layout::<TransformBindGroup>(),
                    storage.get_bind_group_layout::<CameraBindGroup>(),
                ],
                push_constant_ranges: &[],
            }),
            vertex_layouts: &[MeshVertex::layout()],
            vertex_entry_point: "vs_main",
            color_targets: Some(&[Some(ColorTargetState {
                format: renderer.surface_format(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })]),
            fragment_entry_point: "fs_main",
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        }
        .build(&renderer);
        let color_pipeline_id = storage.insert_pipeline(color_pipeline);

        let instance_pipeline = PipelineBuilder {
            shader_path: "./shaders/instance.wgsl",
            label: Some("instance_pipeline"),
            layout_descriptor: Some(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[storage.get_bind_group_layout::<CameraBindGroup>()],
                push_constant_ranges: &[],
            }),
            vertex_layouts: &[MeshVertex::layout(), InstanceVertex::layout()],
            vertex_entry_point: "vs_main",
            color_targets: Some(&[Some(ColorTargetState {
                format: renderer.surface_format(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })]),
            fragment_entry_point: "fs_main",
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        }
        .build(&renderer);
        let instance_pipeline_id = storage.insert_pipeline(instance_pipeline);

        let phase = RenderPhase::new(
            const_vec![ColorAttachment {
                view_id: ResourceId::WINDOW_VIEW_ID,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
            },],
            None,
        );

        let camera = GameCamera::new(&renderer, &mut storage, [0.0, 0.0, 5.0]);

        let boxes = Instances::new(&renderer, &mut storage, Quad::new(1.0, 1.0), 2 + 1 + 5 * 7);

        let border = Border::new(
            15.0,
            20.0,
            0.2,
            [0.7, 0.7, 0.7, 1.0],
            [0.0, 0.0, 0.0, 0.0],
            0,
        );
        border.render_sync(&renderer, &storage, &boxes);

        let platform = Platform::new(
            Vector3 {
                x: 0.0,
                y: -8.0,
                z: 0.0,
            },
            2.0,
            0.5,
            [0.9, 0.16, 0.21, 1.0],
            5.0,
            std::mem::size_of::<InstanceUniform>() as u64 * 2,
        );
        platform.render_sync(&renderer, &storage, &boxes);

        let ball = Ball::new(
            &renderer,
            &mut storage,
            Vector3 {
                x: 0.0,
                y: -7.0,
                z: 0.0,
            },
            0.5,
            [0.0, 0.9, 0.18, 1.0],
            Vector2 { x: 2.5, y: 2.5 },
            1.0,
        );

        let mut crate_pack = CratePack::new(
            Vector3 {
                x: 0.0,
                y: 4.0,
                z: 0.0,
            },
            5,
            7,
            1.5,
            1.0,
            0.2,
            0.2,
            [0.5, 0.5, 0.5, 1.0],
            std::mem::size_of::<InstanceUniform>() as u64 * 3,
        );
        crate_pack.render_sync(&renderer, &storage, &boxes);

        Self {
            renderer,
            storage,
            color_pipeline_id,
            instance_pipeline_id,
            box_instances: boxes,
            phase,
            camera,
            border,
            ball,
            platform,
            crate_pack,
        }
    }

    pub fn handle_input(&mut self, key: &Key, state: &ElementState) {
        self.platform.handle_input(key, state);
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.renderer.resize(Some(physical_size));
    }

    pub fn update(&mut self, dt: f32) {
        self.platform.update(&self.border, dt);
        self.ball
            .update(&self.border, &self.platform, &mut self.crate_pack, dt);
    }

    pub fn render_sync(&mut self) {
        self.platform
            .render_sync(&self.renderer, &self.storage, &self.box_instances);
        self.ball.render_sync(&self.renderer, &self.storage);
        self.crate_pack
            .render_sync(&self.renderer, &self.storage, &self.box_instances);
    }

    pub fn render(&mut self) -> bool {
        let current_frame_context = match self.renderer.current_frame() {
            Ok(cfc) => cfc,
            Err(SurfaceError::Lost) => {
                self.renderer.resize(None);
                return true;
            }
            Err(SurfaceError::OutOfMemory) => {
                return false;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return false;
            }
        };

        let current_frame_storage = CurrentFrameStorage {
            storage: &self.storage,
            current_frame_view: current_frame_context.view(),
        };

        let mut encoder = self.renderer.create_encoder();

        let ball_command = self
            .ball
            .render_command(self.color_pipeline_id, self.camera.bind_group.0);
        let boxes_command = self
            .box_instances
            .render_command(self.instance_pipeline_id, self.camera.bind_group.0);
        {
            let mut render_pass = self.phase.render_pass(&mut encoder, &current_frame_storage);
            boxes_command.execute(&mut render_pass, &current_frame_storage);
            ball_command.execute(&mut render_pass, &current_frame_storage);
        }

        let commands = encoder.finish();
        self.renderer.submit(std::iter::once(commands));
        current_frame_context.present();

        true
    }
}
