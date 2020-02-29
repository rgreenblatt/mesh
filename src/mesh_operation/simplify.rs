use crate::mesh::Mesh;
use crate::mesh_operation::Operation;
use clap::Clap;

#[derive(Clap)]
pub struct Simplify {
  faces_to_remove: u32,
}

impl Operation for Simplify {
  fn apply(&self, mesh: &Mesh) -> Mesh {
    Mesh {
      vertices: Vec::new(),
      faces: Vec::new(),
    }
  }
}
