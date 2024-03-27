use winit::{event::ElementState, keyboard::Key};
use zero::{
    cgmath_imports::{Matrix4, Vector3},
    render::{renderer::Renderer, storage::RenderStorage},
    transform::Transform,
};

use crate::{
    border::Border,
    physics::{Collider, Collision, Rectangle},
    rendering::{InstanceUniform, Instances},
};

pub struct Platform {
    position: Vector3<f32>,
    width: f32,
    height: f32,
    color: [f32; 4],
    speed: f32,
    movement: f32,
    instance_buffer_offset: u64,
}

impl Platform {
    pub fn new(
        position: Vector3<f32>,
        width: f32,
        height: f32,
        color: [f32; 4],
        speed: f32,
        instance_buffer_offset: u64,
    ) -> Self {
        Self {
            position,
            width,
            height,
            color,
            speed,
            movement: 0.0,
            instance_buffer_offset,
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
                    self.movement = pressed;
                }
                "d" | "D" => {
                    self.movement = -pressed;
                }
                _ => {}
            }
        }
    }

    #[inline]
    pub fn border(&self) -> Rectangle {
        Rectangle::from_center(self.position.truncate(), self.width, self.height)
    }

    pub fn update(&mut self, border: &Border, dt: f32) {
        self.position.x -= self.movement * self.speed * dt;

        if let Some(collision) = border.collides(self) {
            if 0.0 <= collision.normal.x {
                self.position.x = collision.pos.x + self.width / 2.0;
            } else {
                self.position.x = collision.pos.x - self.width / 2.0;
            }
        }
    }

    pub fn render_sync(&self, renderer: &Renderer, storage: &RenderStorage, boxes: &Instances) {
        let data = InstanceUniform {
            transform: Matrix4::from(&Transform {
                translation: self.position,
                scale: Vector3::new(self.width, self.height, 1.0),
                ..Default::default()
            })
            .into(),
            color: self.color,
            disabled: 0,
        };
        boxes.instance_buffer_handle.update(
            renderer,
            storage,
            self.instance_buffer_offset,
            &[data],
        );
    }
}

impl Collider for Platform {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.border())
    }

    #[inline]
    fn collides(&self, other: &impl Collider) -> Option<Collision> {
        self.border().collides(other)
    }
}
