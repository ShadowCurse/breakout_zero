use winit::{event::ElementState, keyboard::Key};
use zero::{
    cgmath_imports::Vector3,
    mesh::MeshRenderCommand,
    render::{
        renderer::Renderer,
        storage::{RenderStorage, ResourceId},
    },
};

use crate::{
    border::Border,
    physics::{Collider, Collision, Rectangle},
    GameObject,
};

pub struct Platform {
    game_object: GameObject,
    speed: f32,

    movement: f32,
}

impl Platform {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        position: Vector3<f32>,
        width: f32,
        height: f32,
        color: [f32; 4],
        speed: f32,
    ) -> Self {
        Self {
            game_object: GameObject::new(renderer, storage, width, height, color, position),
            speed,
            movement: 0.0,
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

    pub fn update(&mut self, border: &Border, dt: f32) {
        self.game_object.transform.translation.x -= self.movement * self.speed * dt;

        if let Some(collision) = border.collides(self) {
            if 0.0 <= collision.normal.x {
                self.game_object.transform.translation.x =
                    collision.pos.x + self.game_object.quad.width / 2.0;
            } else {
                self.game_object.transform.translation.x =
                    collision.pos.x - self.game_object.quad.width / 2.0;
            }
        }
    }

    pub fn render_sync(&self, renderer: &Renderer, storage: &RenderStorage) {
        self.game_object.update_transform(renderer, storage);
    }

    pub fn render_command(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> MeshRenderCommand {
        self.game_object.command(pipeline_id, camera_bind_group)
    }
}

impl Collider for Platform {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.game_object.rect())
    }

    #[inline]
    fn collides(&self, other: &impl Collider) -> Option<Collision> {
        self.game_object.rect().collides(other)
    }
}
