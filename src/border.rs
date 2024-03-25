use zero::{
    cgmath_imports::{Vector2, Vector3},
    mesh::MeshRenderCommand,
    render::{
        renderer::Renderer,
        storage::{RenderStorage, ResourceId},
    },
    shapes::Quad,
};

use crate::{
    physics::{Collider, Collision, Rectangle},
    GameObject,
};

pub struct Border {
    outer_rect: GameObject,
    inner_rect: GameObject,
}

impl Border {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        width: f32,
        height: f32,
        thickness: f32,
        border_color: [f32; 4],
        inner_color: [f32; 4],
    ) -> Self {
        Self {
            outer_rect: GameObject::new(
                renderer,
                storage,
                Quad::new(width, height),
                width,
                height,
                border_color,
                Vector3::new(0.0, 0.0, -0.1),
            ),
            inner_rect: GameObject::new(
                renderer,
                storage,
                Quad::new(width - thickness, height - thickness),
                width - thickness,
                height - thickness,
                inner_color,
                Vector3::new(0.0, 0.0, -0.01),
            ),
        }
    }

    pub fn render_commands(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> [MeshRenderCommand; 2] {
        [
            self.outer_rect.command(pipeline_id, camera_bind_group),
            self.inner_rect.command(pipeline_id, camera_bind_group),
        ]
    }
}

impl Collider for Border {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.inner_rect.rect())
    }

    fn collides(&self, other: &impl Collider) -> Option<Collision> {
        let this_rect = self.rect();
        let other_rect = other.rect();

        if this_rect.is_none() || other_rect.is_none() {
            return None;
        }

        let this_rect = this_rect.unwrap();
        let other_rect = other_rect.unwrap();

        if other_rect.left() < this_rect.left() {
            Some(Collision {
                pos: Vector2 {
                    x: this_rect.left(),
                    y: other_rect.pos().y,
                },
                normal: Vector2 { x: 1.0, y: 0.0 },
            })
        } else if this_rect.right() < other_rect.right() {
            Some(Collision {
                pos: Vector2 {
                    x: this_rect.right(),
                    y: other_rect.pos().y,
                },
                normal: Vector2 { x: -1.0, y: 0.0 },
            })
        } else if other_rect.top() < this_rect.top() {
            Some(Collision {
                pos: Vector2 {
                    x: other_rect.pos().x,
                    y: this_rect.top(),
                },
                normal: Vector2 { x: 0.0, y: 1.0 },
            })
        } else if this_rect.bot() < other_rect.bot() {
            Some(Collision {
                pos: Vector2 {
                    x: other_rect.pos().x,
                    y: this_rect.bot(),
                },
                normal: Vector2 { x: 0.0, y: -1.0 },
            })
        } else {
            None
        }
    }
}
