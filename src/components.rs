use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_platform::collections::HashMap;

#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Component for storing position for physics.
///
/// Multiply by `TILE_SIZE` to obtain position for rendering.
#[derive(Component, Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[require(Movement)]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Position(pub Vec2);

/// Do not modify velocity directly
/// Instead use apply_force to change velocity
#[derive(Component, Default, Clone, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Movement {
    /// Displacement of an object per frame.
    ///
    /// Do not modify directly. Instead use `apply_force()` to change velocity.
    pub velocity: Vec2,
    /// List of forces that act upon an object. Used to calculate the velocity.
    ///
    /// Use `apply_force()` to add a force. Remove forces directly with `HashMap.remove()`
    pub forces: HashMap<String, Force>,
    /// Scalar by which `Force`s that are inactive will be damped with.
    pub damping: Vec2,
}

impl Movement {
    pub const MAX_VELOCITY: f32 = 256.0;

    pub fn damped(damping: Vec2) -> Self {
        Self {
            damping,
            ..Default::default()
        }
    }

    pub fn apply_force(&mut self, partial: PartialForce) {
        let id = partial.id.clone();

        let new_force = match self.forces.get(&id) {
            Some(old_force) => old_force.mix(&partial),
            None => partial.into(),
        };

        self.forces.insert(id, new_force);
    }

    pub fn apply_damping(&mut self, dt: f32) {
        for (_, force) in &mut self.forces {
            if !force.active {
                force.force *= self.damping * dt;
            }
        }
    }
}

/// Collider represented by a rectangle with rounded corners
#[derive(Component, Clone, Copy, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Collider {
    pub size: Vec2,
    pub radius: f32,
    pub ctype: ColliderType,
}

impl Collider {
    pub const DEFAULT_RADIUS: f32 = 0.2;

    pub const fn new(size: Vec2, radius: f32, ctype: ColliderType) -> Self {
        let diameter = radius * 2.0;

        debug_assert!(size.x >= diameter);
        debug_assert!(size.y >= diameter);

        Self {
            size,
            radius,
            ctype,
        }
    }

    pub const fn rect(size: Vec2, ctype: ColliderType) -> Self {
        Self {
            size,
            radius: 0.0,
            ctype,
        }
    }

    pub const fn circle(radius: f32, ctype: ColliderType) -> Self {
        Self {
            size: Vec2::splat(radius * 2.0),
            radius,
            ctype,
        }
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self::new(Vec2::ONE, Self::DEFAULT_RADIUS, ColliderType::default())
    }
}

#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub enum ColliderType {
    /// Collider with no collision response (default)
    #[default]
    Sensor,
    /// Collider that does not move when it collides
    Static,
    /// Collider that get pushed away on collision based on mass
    /// Mass must be finite and non-zero
    Dynamic(f32),
}

#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct PartialForce {
    pub id: String,
    pub force: Option<Vec2>,
    pub active: Option<bool>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct Force {
    pub id: String,
    pub force: Vec2,
    pub active: bool,
}

impl Force {
    pub const DEFAULT_NAME: &str = "default_force";

    pub fn mix(&self, partial: &PartialForce) -> Self {
        Self {
            id: self.id.clone(),
            force: partial.force.unwrap_or(self.force),
            active: partial.active.unwrap_or(self.active),
        }
    }
}

impl Default for Force {
    fn default() -> Self {
        Self {
            id: Self::DEFAULT_NAME.to_string(),
            force: Vec2::ZERO,
            active: false,
        }
    }
}

impl From<PartialForce> for Force {
    fn from(value: PartialForce) -> Self {
        Self {
            id: value.id,
            force: value.force.unwrap_or(Vec2::ZERO),
            active: value.active.unwrap_or(false),
        }
    }
}

impl std::ops::Mul<Vec2> for Force {
    type Output = Self;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Self {
            id: self.id,
            force: self.force * rhs,
            active: self.active,
        }
    }
}

impl PartialEq for Force {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Force {}

impl std::hash::Hash for Force {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
