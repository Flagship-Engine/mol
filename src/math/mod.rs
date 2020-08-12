//! Math-related constructs used by the other modules.

#![macro_use]

mod mat3;
mod vec2;
mod vec3;

pub(crate) use mat3::Mat3;
pub use vec2::Vec2;
pub use vec3::Vec3;
