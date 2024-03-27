use zero::{
    cgmath_imports::{Matrix4, Vector2, Vector3},
    render::{renderer::Renderer, storage::RenderStorage},
    transform::Transform,
};

use crate::{
    physics::{Collider, Collision, Rectangle},
    InstanceUniform, Instances,
};

pub struct Border {
    width: f32,
    height: f32,
    thickness: f32,
    border_color: [f32; 4],
    inner_color: [f32; 4],
    instance_buffer_offset: u64,
}

impl Border {
    pub fn new(
        width: f32,
        height: f32,
        thickness: f32,
        border_color: [f32; 4],
        inner_color: [f32; 4],
        instance_buffer_offset: u64,
    ) -> Self {
        Self {
            width,
            height,
            thickness,
            border_color,
            inner_color,
            instance_buffer_offset,
        }
    }

    #[inline]
    pub fn border(&self) -> Rectangle {
        Rectangle::from_center(Vector2::new(0.0, 0.0), self.width, self.height)
    }

    pub fn render_sync(&self, renderer: &Renderer, storage: &RenderStorage, boxes: &Instances) {
        let data = [
            InstanceUniform {
                transform: Matrix4::from(&Transform {
                    translation: Vector3::new(0.0, 0.0, -0.1),
                    scale: Vector3::new(self.width, self.height, 1.0),
                    ..Default::default()
                })
                .into(),
                color: self.border_color,
                disabled: 0,
            },
            InstanceUniform {
                transform: Matrix4::from(&Transform {
                    translation: Vector3::new(0.0, 0.0, -0.01),
                    scale: Vector3::new(
                        self.width - self.thickness,
                        self.height - self.thickness,
                        1.0,
                    ),
                    ..Default::default()
                })
                .into(),
                color: self.inner_color,
                disabled: 0,
            },
        ];
        boxes.instance_buffer_handle.update(
            renderer,
            storage,
            self.instance_buffer_offset,
            &data,
        );
    }
}

impl Collider for Border {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.border())
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
