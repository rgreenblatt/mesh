use crate::mesh_operation::Operation;
use crate::DataStructure;
use clap::Clap;

#[derive(Clap)]
pub struct Remesh {
  iterations: u32,
  smoothing_weight: f32,
}

impl Operation for Remesh {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {}
}
