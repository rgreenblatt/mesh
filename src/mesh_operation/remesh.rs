use crate::mesh::Mesh;
use crate::mesh_operation::Operation;
use clap::Clap;

#[derive(Clap)]
pub struct Remesh {
  iterations: u32,
  smoothing_weight: f32,
}

impl Operation for Remesh {
  fn apply(&self, mesh: &Mesh) -> Mesh {
    Mesh {
      vertices: Vec::new(),
      faces: Vec::new(),
    }
  }
}
