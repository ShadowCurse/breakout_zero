use wgpu::StoreOp;
use winit::{dpi::PhysicalSize, event::ElementState, keyboard::Key, window::Window};
use zero::{const_vec, impl_simple_buffer, prelude::*};

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
    mesh: Mesh,
    mesh_id: ResourceId,

    material: ColorMaterial,
    material_handle: ColorMaterialHandle,
    material_bind_group: ColorMaterialBindGroup,

    transform: Transform,
    transform_handle: TransformHandle,
    transform_bind_group: TransformBindGroup,
}

impl GameObject {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        width: f32,
        height: f32,
        color: [f32; 4],
        position: [f32; 3],
    ) -> Self {
        let mesh: Mesh = Quad::new(width, height).into();
        let mesh_id = storage.insert_mesh(mesh.build(renderer));

        let material = ColorMaterial { color };
        let material_handle = ColorMaterialHandle::new(storage, material.build(renderer));
        let material_bind_group = ColorMaterialBindGroup::new(renderer, storage, &material_handle);

        let transform = Transform {
            translation: position.into(),
            ..Default::default()
        };
        let transform_handle = TransformHandle::new(storage, transform.build(renderer));
        let transform_bind_group = TransformBindGroup::new(renderer, storage, &transform_handle);

        Self {
            mesh,
            mesh_id,
            material,
            material_handle,
            material_bind_group,
            transform,
            transform_handle,
            transform_bind_group,
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
}

pub struct Game {
    renderer: Renderer,
    storage: RenderStorage,

    color_pipeline_id: ResourceId,
    depth_texture_id: ResourceId,

    phase: RenderPhase,

    camera: GameCamera,

    ball: GameObject,
    platform: GameObject,
    crates: Vec<GameObject>,

    platform_movement: f32,
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

        let ball = GameObject::new(
            &renderer,
            &mut storage,
            1.0,
            1.0,
            [0.4, 0.9, 0.4, 1.0],
            [0.0, 0.0, 0.0],
        );
        let platform = GameObject::new(
            &renderer,
            &mut storage,
            2.0,
            0.5,
            [0.6, 0.6, 0.6, 1.0],
            [0.0, -1.0, 0.0],
        );

        let center = Vector3::new(0.0, 3.0, 0.0);
        let rows: u32 = 3;
        let cols: u32 = 4;
        let width: f32 = 1.5;
        let height: f32 = 0.8;
        let gap_x: f32 = 0.2;
        let gap_y: f32 = 0.2;
        let bottom_left = center
            - Vector3::new(
                (gap_x + width) / 2.0 * (cols - 1) as f32,
                (gap_y + height) / 2.0 * (rows - 1) as f32,
                0.0,
            );
        let mut crates = vec![];
        for x in 0..cols {
            for y in 0..rows {
                let c = GameObject::new(
                    &renderer,
                    &mut storage,
                    width,
                    height,
                    [0.8, 0.8, 0.8, 1.0],
                    [
                        bottom_left.x + x as f32 * (width + gap_x),
                        bottom_left.y + y as f32 * (height + gap_y),
                        0.0,
                    ],
                );
                crates.push(c);
            }
        }

        Self {
            renderer,
            storage,
            color_pipeline_id,
            depth_texture_id,
            phase,
            camera,
            ball,
            platform,
            crates,
            platform_movement: 0.0,
        }
    }

    pub fn handle_input(&mut self, key: &Key, state: &ElementState) {
        let pressed = if *state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        if let Key::Character(c) = key {
            match c.as_str() {
                "a" | "A" => {
                    self.platform_movement = -pressed;
                }
                "d" | "D" => {
                    self.platform_movement = pressed;
                }
                _ => {}
            }
        }
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.renderer.resize(Some(physical_size));
        self.storage.replace_texture(
            self.depth_texture_id,
            EmptyTexture::new_depth().build(&self.renderer),
        );
    }

    pub fn update(&mut self, dt: f32) {
        self.platform.transform.translation +=
            Vector3::new(self.platform_movement * 10.0 * dt, 0.0, 0.0);
    }

    pub fn render_sync(&mut self) {
        self.platform.transform_handle.update(
            &self.renderer,
            &self.storage,
            &self.platform.transform,
        );
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
            .command(self.color_pipeline_id, self.camera.bind_group.0);
        let platform_command = self
            .platform
            .command(self.color_pipeline_id, self.camera.bind_group.0);
        let crates_commands = self
            .crates
            .iter()
            .map(|c| c.command(self.color_pipeline_id, self.camera.bind_group.0))
            .collect::<Vec<_>>();
        {
            let mut render_pass = self.phase.render_pass(&mut encoder, &current_frame_storage);
            for command in [ball_command, platform_command]
                .iter()
                .chain(crates_commands.iter())
            {
                command.execute(&mut render_pass, &current_frame_storage);
            }
        }

        let commands = encoder.finish();
        self.renderer.submit(std::iter::once(commands));
        current_frame_context.present();

        true
    }
}
