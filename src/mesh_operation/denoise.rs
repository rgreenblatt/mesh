use crate::mesh_operation::Operation;
use crate::DataStructure;

use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::FromIterator;

use clap::Clap;

#[derive(Clap)]
pub struct Denoise {
  iterations: u32,
  sigma_c: f32,
  sigma_s: f32,
  kernel_size: u32,
}

impl Operation for Denoise {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    let mut store = Vec::new();
    let mut neighborhood = HashSet::new();
    let mut new_vertices = HashSet::new();
    let mut neighbors_this_round;

    let mut new_positions = Vec::new();

    for _ in 0..self.iterations {
      let double_var_c = 2.0 * self.sigma_c.powi(2);
      let double_var_s = 2.0 * self.sigma_s.powi(2);

      new_positions.resize(mesh.max_idx_vertices(), None);

      let mut vertex_op = mesh.initial_vertex();

      while let Some(vertex_idx) = vertex_op {
        let normal = mesh.get_vertex_normal(vertex_idx, &mut store);

        neighborhood.clear();

        neighborhood.insert(vertex_idx);
        new_vertices.insert(vertex_idx);

        for _ in 0..self.kernel_size {
          store.clear();

          for other_vertex_idx in new_vertices.iter() {
            mesh.get_vertex_neighbors_append(*other_vertex_idx, &mut store);
          }

          neighbors_this_round = HashSet::from_iter(store.iter().cloned());

          new_vertices = HashSet::from_iter(
            neighbors_this_round.difference(&neighborhood).cloned(),
          );

          neighborhood.extend(&new_vertices);
        }

        let vertex_pos = mesh.get_position(vertex_idx);

        let (sum, normalizer) = neighborhood.iter().fold(
          (0.0, 0.0),
          |(sum, normalizer), neighbor| {
            let neighbor = *neighbor;
            let diff = vertex_pos - mesh.get_position(neighbor);

            let diff_norm = diff.norm();

            let height = normal.dot(&diff);

            let w_c = (-diff_norm.powi(2) / double_var_c).exp();
            let w_s = (-height.powi(2) / double_var_s).exp();

            let w = w_c * w_s;

            (sum + w * height, normalizer + w)
          },
        );

        new_positions[vertex_idx as usize] =
          Some(vertex_pos - normal * (sum / normalizer));

        vertex_op = mesh.next_vertex(vertex_idx);
      }

      let iter = new_positions
        .iter()
        .enumerate()
        .filter_map(|(vertex_idx, pos_op)| pos_op.map(|x| (vertex_idx, x)));

      for (vertex_idx, pos) in iter {
        mesh.set_position(vertex_idx.try_into().unwrap(), &pos);
      }
    }
  }
}
