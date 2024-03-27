use zero::{
    const_vec,
    prelude::{
        winit::{dpi::PhysicalSize, event::ElementState, keyboard::Key, window::Window},
        *,
    },
};

use crate::{
    ball::Ball,
    border::Border,
    crates::CratePack,
    platform::Platform,
    rendering::{InstanceUniform, InstanceVertex, Instances},
};

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
pub struct Game<'window> {
    renderer: Renderer<'window>,
    storage: RenderStorage,

    instance_pipeline_id: ResourceId,
    phase: RenderPhase,

    camera: GameCamera,

    box_instances: Instances,

    border: Border,
    ball: Ball,
    platform: Platform,
    crate_pack: CratePack,
}

impl<'window> Game<'window> {
    pub fn new(window: &'window Window) -> Game<'window> {
        let renderer = pollster::block_on(Renderer::new(window));
        let mut storage = RenderStorage::default();

        storage.register_bind_group_layout::<CameraBindGroup>(&renderer);
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

        // 2 instances for border
        // 1 instance for platform
        // 5 * 7 instances for crates
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
            .render_command(self.instance_pipeline_id, self.camera.bind_group.0);
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
