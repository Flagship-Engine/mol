/// A basic 2-dimensional vector.
/// Used by [`OBJ`] to represent texture coordinates.
///
/// [`OBJ`]: /mol/obj/struct.OBJ.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    /// Constructs a new 2-dimensional vector using the provided values.
    #[inline(always)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline(always)]
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "⟨{}, {}⟩", self.x, self.y)
    }
}
