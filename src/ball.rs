use zero::{
    cgmath_imports::{Matrix4, Vector2, Vector3},
    render::{
        renderer::Renderer,
        storage::{RenderStorage, ResourceId},
    },
    shapes::Circle,
    transform::Transform,
};

use crate::{
    border::Border,
    crates::CratePack,
    physics::{Collider, Collision, Rectangle},
    platform::Platform,
    InstanceUniform, Instances, InstancesRenderCommand,
};

pub struct Ball {
    // game_object: GameObject,
    instance: Instances,

    transform: Transform,
    radius: f32,
    color: [f32; 4],
    velocity: Vector2<f32>,
    speed: f32,
}

impl Ball {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        translation: Vector3<f32>,
        radius: f32,
        color: [f32; 4],
        velocity: Vector2<f32>,
        speed: f32,
    ) -> Self {
        let mesh = Circle::new(radius, 50);
        let instance = Instances::new(renderer, storage, mesh, 1);
        let transform = Transform {
            translation,
            ..Default::default()
        };
        Self {
            instance,
            transform,
            radius,
            color,
            velocity,
            speed,
        }
    }

    #[inline]
    pub fn border(&self) -> Rectangle {
        Rectangle::from_center(
            self.transform.translation.truncate(),
            self.radius * 2.0,
            self.radius * 2.0,
        )
    }

    pub fn update(
        &mut self,
        border: &Border,
        platform: &Platform,
        crate_pack: &mut CratePack,
        dt: f32,
    ) {
        self.transform.translation.x += self.velocity.x * self.speed * dt;
        self.transform.translation.y += self.velocity.y * self.speed * dt;

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
        let data = InstanceUniform {
            transform: Matrix4::from(&self.transform).into(),
            color: self.color,
            disabled: 0,
        };
        self.instance
            .instance_buffer_handle
            .update(renderer, storage, 0, &[data]);
    }

    pub fn render_command(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> InstancesRenderCommand {
        self.instance.render_command(pipeline_id, camera_bind_group)
    }
}

impl Collider for Ball {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.border())
    }
}
