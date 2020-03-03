use crate::DataStructure;

pub trait Operation {
  fn apply<D: DataStructure>(&self, mesh: &mut D);
}

mod denoise;
mod noise;
mod remesh;
mod simplify;
mod subdivide;

pub use denoise::Denoise;
pub use noise::Noise;
pub use remesh::Remesh;
pub use simplify::Simplify;
pub use subdivide::Subdivide;
