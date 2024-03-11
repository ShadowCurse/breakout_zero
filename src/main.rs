use wgpu::StoreOp;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};
use zero::{const_vec, prelude::*};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = pollster::block_on(Renderer::new(&window));
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

    let mut camera = Camera::new(
        (-15.0, 2.0, 0.0),
        Deg(0.0),
        Deg(0.0),
        renderer.size().width,
        renderer.size().height,
        Deg(90.0),
        0.1,
        100.0,
    );
    let camera_handle = CameraHandle::new(&mut storage, camera.build(&renderer));
    let camera_bind_group = CameraBindGroup::new(&renderer, &mut storage, &camera_handle);

    let ball_mesh: Mesh = Quad::new(1.0, 1.0).into();
    let ball_id = storage.insert_mesh(ball_mesh.build(&renderer));

    let ball_transform = Transform {
        translation: (0.0, 0.0, 0.0).into(),
        // quads are in x/y plane
        // camera looks at z/y plane
        rotation: Quaternion::from_angle_y(Deg(-90.0)),
        ..Default::default()
    };
    let ball_transform_handle = TransformHandle::new(&mut storage, ball_transform.build(&renderer));
    let ball_transform_bind_group =
        TransformBindGroup::new(&renderer, &mut storage, &ball_transform_handle);

    let platform_mesh: Mesh = Quad::new(2.0, 0.5).into();
    let platform_id = storage.insert_mesh(platform_mesh.build(&renderer));

    let mut platform_transform = Transform {
        translation: (0.0, -1.0, 0.0).into(),
        rotation: Quaternion::from_angle_y(Deg(-90.0)),
        ..Default::default()
    };
    let platform_transform_handle =
        TransformHandle::new(&mut storage, platform_transform.build(&renderer));
    let platform_transform_bind_group =
        TransformBindGroup::new(&renderer, &mut storage, &platform_transform_handle);

    let grey_material = ColorMaterial {
        ambient: [0.4, 0.4, 0.4],
        diffuse: [0.6, 0.6, 0.6],
        specular: [1.0, 1.0, 1.0],
        shininess: 32.0,
    };
    let grey_material_handle =
        ColorMaterialHandle::new(&mut storage, grey_material.build(&renderer));
    let grey_material_bind_group =
        ColorMaterialBindGroup::new(&renderer, &mut storage, &grey_material_handle);

    let green_material = ColorMaterial {
        ambient: [0.4, 0.9, 0.4],
        diffuse: [0.4, 0.9, 0.4],
        specular: [0.1, 0.1, 0.1],
        shininess: 1.0,
    };
    let green_material_handle =
        ColorMaterialHandle::new(&mut storage, green_material.build(&renderer));
    let green_material_bind_group =
        ColorMaterialBindGroup::new(&renderer, &mut storage, &green_material_handle);

    let mut platform_movement: f32 = 0.0;

    let mut last_render_time = std::time::Instant::now();
    _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: key,
                            state,
                            ..
                        },
                    ..
                } => {
                    // handle input
                    let pressed = if *state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    };
                    match key {
                        Key::Named(NamedKey::Escape) => target.exit(),
                        Key::Character(c) => match c.as_str() {
                            "a" | "A" => {
                                platform_movement = -pressed;
                            }
                            "d" | "D" => {
                                platform_movement = pressed;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    camera.resize(physical_size.width, physical_size.height);
                    renderer.resize(Some(*physical_size));
                    storage.replace_texture(
                        depth_texture_id,
                        EmptyTexture::new_depth().build(&renderer),
                    );
                }
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    // game update
                    platform_transform.translation +=
                        Vector3::new(0.0, 0.0, platform_movement * 10.0 * dt.as_secs_f32());

                    // render update
                    platform_transform_handle.update(&renderer, &storage, &platform_transform);

                    // render
                    let current_frame_context = match renderer.current_frame() {
                        Ok(cfc) => cfc,
                        Err(SurfaceError::Lost) => {
                            renderer.resize(None);
                            return;
                        }
                        Err(SurfaceError::OutOfMemory) => {
                            target.exit();
                            return;
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                            return;
                        }
                    };

                    let current_frame_storage = CurrentFrameStorage {
                        storage: &storage,
                        current_frame_view: current_frame_context.view(),
                    };

                    let mut encoder = renderer.create_encoder();

                    let ball = MeshRenderCommand {
                        pipeline_id: color_pipeline_id,
                        mesh_id: ball_id,
                        index_slice: None,
                        vertex_slice: None,
                        scissor_rect: None,
                        bind_groups: const_vec![
                            green_material_bind_group.0,
                            ball_transform_bind_group.0,
                            camera_bind_group.0,
                        ],
                    };
                    let platform = MeshRenderCommand {
                        pipeline_id: color_pipeline_id,
                        mesh_id: platform_id,
                        index_slice: None,
                        vertex_slice: None,
                        scissor_rect: None,
                        bind_groups: const_vec![
                            grey_material_bind_group.0,
                            platform_transform_bind_group.0,
                            camera_bind_group.0,
                        ],
                    };
                    {
                        let mut render_pass =
                            phase.render_pass(&mut encoder, &current_frame_storage);
                        for command in [ball, platform] {
                            command.execute(&mut render_pass, &current_frame_storage);
                        }
                    }

                    let commands = encoder.finish();
                    renderer.submit(std::iter::once(commands));
                    current_frame_context.present();
                }
                _ => {}
            },
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}
