use crate::mesh_operation::Operation;
use crate::DataStructure;

use rand_distr::{Distribution, Normal};

use clap::Clap;

#[derive(Clap)]
pub struct Noise {
  sigma: f32,
}

impl Operation for Noise {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    let mut vertex_op = mesh.initial_vertex();
    let mut store = Vec::new();

    let dist =
      Normal::new(0.0, self.sigma).expect("distribution should be valid");

    while let Some(vertex_idx) = vertex_op {
      let normal = mesh.get_vertex_normal(vertex_idx, &mut store);

      let noise = normal * dist.sample(&mut rand::thread_rng());

      let new_position = mesh.get_position(vertex_idx) + noise;
      mesh.set_position(vertex_idx, &new_position);

      vertex_op = mesh.next_vertex(vertex_idx);
    }
  }
}
