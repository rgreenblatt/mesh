use crate::mesh::Mesh;

pub trait Operation {
  fn apply(&self, mesh: &Mesh) -> Mesh;
}

mod denoise;
mod remesh;
mod simplify;
mod subdivide;

pub use denoise::Denoise;
pub use remesh::Remesh;
pub use simplify::Simplify;
pub use subdivide::Subdivide;
