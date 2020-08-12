/// A basic 3-dimensional vector.
/// Used by [`OBJ`] to represent vertex positions and normals.
///
/// [`OBJ`]: /mol/obj/struct.OBJ.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const UNIT_Y: Vec3 = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    /// Constructs a new 3-dimensional vector using the provided values.
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculates the dot product between two 3-dimensional vectors.
    #[inline(always)]
    pub(crate) fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculates the squared magnitude of the vector.
    #[inline(always)]
    pub(crate) fn squared_magnitude(&self) -> f32 {
        self.dot(self)
    }

    /// Normalizes the vector.
    #[inline(always)]
    pub(crate) fn normalize(&mut self) {
        let mag = self.squared_magnitude();
        if mag == 0.0 {
            return;
        }
        let mag = mag.sqrt();
        self.x /= mag;
        self.y /= mag;
        self.z /= mag;
    }

    /// Calculates the cross product between two 3-dimensional vectors.
    pub fn cross(&self, other: &Vec3) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.z - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub(crate) fn unit_align(u1: &Self, u2: &Self) -> super::Mat3 {
        let Vec3 { x, y, z } = u1.cross(u2);

        let c = u1.dot(u2);
        let k = 1.0 / (1.0 + c);

        super::Mat3::new([
            (x * x * k) + c,
            (y * x * k) - z,
            (z * x * k) + y,
            (x * y * k) + z,
            (y * y * k) + c,
            (z * y * k) - x,
            (x * z * k) - y,
            (y * z * k) + x,
            (z * z * k) + c,
        ])
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline(always)]
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self { x, y, z }
    }
}
impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "⟨{}, {}, {}⟩", self.x, self.y, self.z)
    }
}
