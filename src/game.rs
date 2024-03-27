use wgpu::StoreOp;
use winit::{dpi::PhysicalSize, event::ElementState, keyboard::Key, window::Window};
use zero::{const_vec, impl_simple_buffer, prelude::*};

use crate::{
    ball::Ball,
    border::Border,
    crates::{CrateInstanceVertex, CratePack, CratesBindGroup},
    physics::Rectangle,
    platform::Platform,
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

pub struct Game {
    renderer: Renderer,
    storage: RenderStorage,

    color_pipeline_id: ResourceId,
    crates_pipeline_id: ResourceId,
    depth_texture_id: ResourceId,

    phase: RenderPhase,

    camera: GameCamera,

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
        storage.register_bind_group_layout::<CratesBindGroup>(&renderer);

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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        }
        .build(&renderer);
        let color_pipeline_id = storage.insert_pipeline(color_pipeline);

        let crates_pipeline = PipelineBuilder {
            shader_path: "./shaders/crates.wgsl",
            label: Some("crates_pipeline"),
            layout_descriptor: Some(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[storage.get_bind_group_layout::<CameraBindGroup>()],
                push_constant_ranges: &[],
            }),
            vertex_layouts: &[MeshVertex::layout(), CrateInstanceVertex::layout()],
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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        }
        .build(&renderer);
        let crates_pipeline_id = storage.insert_pipeline(crates_pipeline);

        let depth_texture_id = storage.insert_texture(EmptyTexture::new_depth().build(&renderer));

        let phase = RenderPhase::new(
            const_vec![ColorAttachment {
                view_id: ResourceId::WINDOW_VIEW_ID,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
            },],
            Some(DepthStencil {
                view_id: depth_texture_id,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        );

        let camera = GameCamera::new(&renderer, &mut storage, [0.0, 0.0, 5.0]);

        let border = Border::new(
            &renderer,
            &mut storage,
            15.0,
            20.0,
            0.2,
            [0.7, 0.7, 0.7, 1.0],
            [0.0, 0.0, 0.0, 0.0],
        );

        let platform = Platform::new(
            &renderer,
            &mut storage,
            Vector3 {
                x: 0.0,
                y: -8.0,
                z: 0.0,
            },
            2.0,
            0.5,
            [0.9, 0.16, 0.21, 1.0],
            5.0,
        );

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

        let crate_pack = CratePack::new(
            &renderer,
            &mut storage,
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
        );

        Self {
            renderer,
            storage,
            color_pipeline_id,
            crates_pipeline_id,
            depth_texture_id,
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
        self.storage.replace_texture(
            self.depth_texture_id,
            EmptyTexture::new_depth().build(&self.renderer),
        );
    }

    pub fn update(&mut self, dt: f32) {
        self.platform.update(&self.border, dt);
        self.ball
            .update(&self.border, &self.platform, &mut self.crate_pack, dt);
    }

    pub fn render_sync(&mut self) {
        self.platform.render_sync(&self.renderer, &self.storage);
        self.ball.render_sync(&self.renderer, &self.storage);
        self.crate_pack.render_sync(&self.renderer, &self.storage);
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

        let border_commands = self
            .border
            .render_commands(self.color_pipeline_id, self.camera.bind_group.0);
        let ball_command = self
            .ball
            .render_command(self.color_pipeline_id, self.camera.bind_group.0);
        let platform_command = self
            .platform
            .render_command(self.color_pipeline_id, self.camera.bind_group.0);
        let crates_command = self
            .crate_pack
            .render_commands(self.crates_pipeline_id, self.camera.bind_group.0);
        {
            let mut render_pass = self.phase.render_pass(&mut encoder, &current_frame_storage);
            for command in border_commands
                .iter()
                .chain([ball_command, platform_command].iter())
            {
                command.execute(&mut render_pass, &current_frame_storage);
            }
            crates_command.execute(&mut render_pass, &current_frame_storage);
        }

        let commands = encoder.finish();
        self.renderer.submit(std::iter::once(commands));
        current_frame_context.present();

        true
    }
}
