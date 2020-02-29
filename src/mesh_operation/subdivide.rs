use crate::DataStructure;
use crate::mesh_operation::Operation;
use clap::Clap;

#[derive(Clap)]
pub struct Subdivide {
  iterations: u32,
}

impl Operation for Subdivide {
  fn apply<D : DataStructure>(&self, mesh: &mut D) {
    mesh.flip_edge(&mesh.initial_edge())
  }
}
