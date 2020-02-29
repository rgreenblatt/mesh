use crate::mesh::Mesh;
use crate::mesh_operation::Operation;
use clap::Clap;

#[derive(Clap)]
pub struct Subdivide {
  iterations: u32,
}

impl Operation for Subdivide {
  fn apply(&self, mesh: &Mesh) -> Mesh {
    Mesh {
      vertices: Vec::new(),
      faces: Vec::new(),
    }
  }
}
