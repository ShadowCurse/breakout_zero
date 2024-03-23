use zero::cgmath_imports::Vector2;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    #[inline]
    pub fn from_center(center: Vector2<f32>, width: f32, height: f32) -> Self {
        Self {
            x: center.x - width / 2.0,
            y: center.y - height / 2.0,
            width,
            height,
        }
    }

    #[inline]
    pub fn pos(&self) -> Vector2<f32> {
        Vector2 {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    #[inline]
    pub fn bot(&self) -> f32 {
        self.y + self.height
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

// Represents collision between colliders
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collision {
    pub pos: Vector2<f32>,
    pub normal: Vector2<f32>,
}

// Trait for determining collison
pub trait Collider {
    fn rect(&self) -> Option<Rectangle>;
    fn collides(&self, _other: &impl Collider) -> Option<Collision> {
        None
    }
    fn collides_mut(&mut self, _other: &impl Collider) -> Option<Collision> {
        None
    }
}

impl Collider for Rectangle {
    #[inline]
    fn rect(&self) -> Option<Rectangle> {
        Some(*self)
    }

    fn collides(&self, other: &impl Collider) -> Option<Collision> {
        let this_rect = self.rect();
        let other_rect = other.rect();

        if this_rect.is_none() || other_rect.is_none() {
            return None;
        }

        let this_rect = this_rect.unwrap();
        let other_rect = other_rect.unwrap();

        if this_rect == other_rect {
            return None;
        }

        let dx = other_rect.pos().x - this_rect.pos().x;
        let px = (other_rect.width + this_rect.width) / 2.0 - dx.abs();
        if px <= 0.0 {
            return None;
        }

        let dy = other_rect.pos().y - this_rect.pos().y;
        let py = (other_rect.height + this_rect.height) / 2.0 - dy.abs();
        if py <= 0.0 {
            return None;
        }

        if px < py {
            let sign = dx.signum();
            Some(Collision {
                pos: Vector2 {
                    x: this_rect.pos().x + this_rect.width / 2.0 * sign,
                    y: other_rect.pos().y,
                },
                normal: Vector2 { x: sign, y: 0.0 },
            })
        } else {
            let sign = dy.signum();
            Some(Collision {
                pos: Vector2 {
                    x: other_rect.pos().x,
                    y: this_rect.pos().y + this_rect.height / 2.0 * sign,
                },
                normal: Vector2 { x: 0.0, y: sign },
            })
        }
    }
}
