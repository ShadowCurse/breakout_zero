use zero::{
    cgmath_imports::{Vector2, Vector3},
    mesh::MeshRenderCommand,
    render::{
        renderer::Renderer,
        storage::{RenderStorage, ResourceId},
    },
    shapes::Circle,
};

use crate::{
    border::Border,
    crates::CratePack,
    physics::{Collider, Collision, Rectangle},
    platform::Platform,
    GameObject,
};

pub struct Ball {
    game_object: GameObject,
    velocity: Vector2<f32>,
    speed: f32,
}

impl Ball {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        position: Vector3<f32>,
        radius: f32,
        color: [f32; 4],
        velocity: Vector2<f32>,
        speed: f32,
    ) -> Self {
        Self {
            game_object: GameObject::new(
                renderer,
                storage,
                Circle::new(radius, 50),
                radius * 2.0,
                radius * 2.0,
                color,
                position,
            ),
            velocity,
            speed,
        }
    }

    pub fn update(
        &mut self,
        border: &Border,
        platform: &Platform,
        crate_pack: &mut CratePack,
        dt: f32,
    ) {
        self.game_object.transform.translation.x += self.velocity.x * self.speed * dt;
        self.game_object.transform.translation.y += self.velocity.y * self.speed * dt;

        self.check_collision(border);
        self.check_collision(platform);
        self.check_collision_mut(crate_pack);
    }

    fn check_collision(&mut self, collider: &impl Collider) {
        if let Some(collision) = collider.collides(self) {
            self.handle_collision(collision);
        }
    }
    fn check_collision_mut(&mut self, collider: &mut impl Collider) {
        if let Some(collision) = collider.collides_mut(self) {
            self.handle_collision(collision);
        }
    }
    fn handle_collision(&mut self, collision: Collision) {
        if collision.normal.x != 0.0 {
            self.velocity.x *= -1.0;
        }
        if collision.normal.y != 0.0 {
            self.velocity.y *= -1.0;
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

impl Collider for Ball {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.game_object.rect())
    }
}
