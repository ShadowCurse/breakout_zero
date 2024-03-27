use zero::prelude::*;

use crate::{
    physics::{Collider, Collision, Rectangle},
    InstanceUniform, Instances,
};

pub struct Crate {
    transform: Transform,
    color: [f32; 4],
    disabled: bool,
}

impl Crate {
    pub fn new(translation: Vector3<f32>, scale: Vector3<f32>, color: [f32; 4]) -> Self {
        Self {
            transform: Transform {
                translation,
                scale,
                ..Default::default()
            },
            color,
            disabled: false,
        }
    }

    #[inline]
    pub fn rect(&self, rect_width: f32, rect_height: f32) -> Rectangle {
        Rectangle::from_center(
            self.transform.translation.truncate(),
            rect_width,
            rect_height,
        )
    }
}

pub struct CratePack {
    pub crates: Vec<Crate>,
    pub rect_width: f32,
    pub rect_height: f32,
    pub need_sync: bool,

    pub instance_buffer_offset: u64,
}

impl CratePack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Vector3<f32>,
        rows: u32,
        cols: u32,
        width: f32,
        height: f32,
        gap_x: f32,
        gap_y: f32,
        color: [f32; 4],
        instance_buffer_offset: u64,
    ) -> Self {
        let bottom_left = center
            - Vector3::new(
                (gap_x + width) / 2.0 * (cols - 1) as f32,
                (gap_y + height) / 2.0 * (rows - 1) as f32,
                0.0,
            );
        let mut crates = vec![];
        for x in 0..cols {
            for y in 0..rows {
                let c = Crate::new(
                    Vector3::new(
                        bottom_left.x + x as f32 * (width + gap_x),
                        bottom_left.y + y as f32 * (height + gap_y),
                        0.0,
                    ),
                    Vector3::new(width, height, 1.0),
                    color,
                );
                crates.push(c);
            }
        }

        Self {
            crates,
            rect_width: width,
            rect_height: height,
            need_sync: true,
            instance_buffer_offset,
        }
    }

    pub fn render_sync(&mut self, renderer: &Renderer, storage: &RenderStorage, boxes: &Instances) {
        if self.need_sync {
            let data = self
                .crates
                .iter()
                .map(|c| InstanceUniform {
                    transform: Matrix4::from(&c.transform).into(),
                    color: c.color,
                    disabled: c.disabled.into(),
                })
                .collect::<Vec<_>>();
            boxes.box_instance_buffer_handle.update(
                renderer,
                storage,
                self.instance_buffer_offset,
                &data,
            );
            self.need_sync = false;
        }
    }
}

impl Collider for CratePack {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        None
    }

    #[inline]
    fn collides_mut(&mut self, other: &impl Collider) -> Option<Collision> {
        for c in self.crates.iter_mut() {
            if !c.disabled {
                let crate_rect = c.rect(self.rect_width, self.rect_height);
                if let Some(collision) = crate_rect.collides(other) {
                    c.disabled = true;
                    self.need_sync = true;
                    return Some(collision);
                }
            }
        }
        None
    }
}
