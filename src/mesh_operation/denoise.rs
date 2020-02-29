use crate::mesh::Mesh;
use crate::mesh_operation::Operation;
use clap::Clap;

#[derive(Clap)]
pub struct Denoise {
  iterations: u32,
  sigma_c: f32,
  sigma_s: f32,
  kernel_size: u32,
}

impl Operation for Denoise {
  fn apply(&self, mesh: &Mesh) -> Mesh {
    Mesh {
      vertices: Vec::new(),
      faces: Vec::new(),
    }
  }
}
