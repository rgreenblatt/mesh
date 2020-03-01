use crate::mesh_operation::Operation;
use crate::DataStructure;
use clap::Clap;

#[derive(Clap)]
pub struct Simplify {
  faces_to_remove: u32,
}

impl Operation for Simplify {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {}
}
