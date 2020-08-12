use super::Vec3;

#[derive(Debug)]
pub(crate) struct Mat3([f32; 3 * 3]);

impl Mat3 {
    pub fn new(data: [f32; 3 * 3]) -> Self {
        Self(data)
    }
}

impl std::ops::Mul<Vec3> for &Mat3 {
    type Output = Vec3;

    fn mul(self, Vec3 { x, y, z }: Vec3) -> Vec3 {
        Vec3 {
            x: x * self.0[0] + y * self.0[1] + z * self.0[2],
            y: x * self.0[3] + y * self.0[4] + z * self.0[5],
            z: x * self.0[6] + y * self.0[7] + z * self.0[8],
        }
    }
}
