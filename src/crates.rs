use zero::{
    cgmath_imports::Vector3,
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

pub struct Crate {
    game_object: GameObject,
    disabled: bool,
}

impl Crate {
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        position: Vector3<f32>,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            game_object: GameObject::new(
                renderer,
                storage,
                Quad::new(width, height),
                width,
                height,
                color,
                position,
            ),
            disabled: false,
        }
    }

    #[inline]
    pub fn disabled(&mut self) -> bool {
        self.disabled
    }

    #[inline]
    pub fn disable(&mut self) {
        self.disabled = true;
    }

    #[inline]
    pub fn render_command(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> Option<MeshRenderCommand> {
        if !self.disabled {
            Some(self.game_object.command(pipeline_id, camera_bind_group))
        } else {
            None
        }
    }
}

impl Collider for Crate {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(self.game_object.rect())
    }

    #[inline]
    fn collides(&self, other: &impl Collider) -> Option<Collision> {
        self.game_object.rect().collides(other)
    }
}

pub struct CratePack {
    crates: Vec<Crate>,
}

impl CratePack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        renderer: &Renderer,
        storage: &mut RenderStorage,
        center: Vector3<f32>,
        rows: u32,
        cols: u32,
        width: f32,
        height: f32,
        gap_x: f32,
        gap_y: f32,
        color: [f32; 4],
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
                    renderer,
                    storage,
                    Vector3::new(
                        bottom_left.x + x as f32 * (width + gap_x),
                        bottom_left.y + y as f32 * (height + gap_y),
                        0.0,
                    ),
                    width,
                    height,
                    color,
                );
                crates.push(c);
            }
        }
        Self { crates }
    }

    #[inline]
    pub fn render_commands(
        &self,
        pipeline_id: ResourceId,
        camera_bind_group: ResourceId,
    ) -> Vec<MeshRenderCommand> {
        self.crates
            .iter()
            .filter_map(|c| c.render_command(pipeline_id, camera_bind_group))
            .collect()
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
            if !c.disabled() {
                if let Some(collision) = c.collides(other) {
                    c.disable();
                    return Some(collision);
                }
            }
        }
        None
    }
}
