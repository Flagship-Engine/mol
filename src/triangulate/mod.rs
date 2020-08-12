//! Everything related to triangulation.

pub mod earclipping;

pub type Tri = [crate::obj::Vertex; 3];
pub type Triangulation = Vec<Tri>;
