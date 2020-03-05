use crate::mesh_operation::Operation;
use crate::DataStructure;
use crate::Vector3;

use clap::Clap;

// use std::collections::HashSet;

#[derive(Clap)]
pub struct Remesh {
  iterations: u32,
  smoothing_weight: f32,
}

impl Operation for Remesh {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    for _ in 0..self.iterations {
      let mut edge_op = mesh.initial_edge();

      let mut total_edge_len = 0.0;
      let mut num_edges = 0;

      let get_edge_len = |mesh: &D, edge_idx| {
        let [l, r] = mesh.get_endpoints(edge_idx);

        (mesh.get_position(l) - mesh.get_position(r)).norm()
      };

      while let Some(edge_idx) = edge_op {
        num_edges += 1;

        total_edge_len += get_edge_len(mesh, edge_idx);

        edge_op = mesh.next_edge(edge_idx);
      }

      let avg_edge_len = total_edge_len / (num_edges as f32);

      let mut to_split = Vec::new();
      let mut to_collapse = Vec::new();
      let mut edge_op = mesh.initial_edge();

      while let Some(edge_idx) = edge_op {
        let edge_len = get_edge_len(mesh, edge_idx);

        if edge_len > (4.0 / 3.0) * avg_edge_len {
          to_split.push(edge_idx);
        } else if edge_len < (4.0 / 5.0) * avg_edge_len {
          to_collapse.push(edge_idx);
        }

        edge_op = mesh.next_edge(edge_idx);
      }

      let midpoint = |mesh: &D, edge_idx| {
        let [l, r] = mesh.get_endpoints(edge_idx);

        (mesh.get_position(l) + mesh.get_position(r)) * 0.5
      };

      for edge_idx in to_split {
        let avg_pos = midpoint(mesh, edge_idx);
        let (new_vertex, _) = mesh.split_edge(edge_idx);
        mesh.set_position(new_vertex, &avg_pos);
      }

      // let mut removed = HashSet::new();
      // let mut store_removed = Vec::new();
      // let mut store_modified = Vec::new();

      // for edge_idx in to_collapse {
      //   if !removed.contains(&edge_idx) {
      //     let avg_pos = midpoint(mesh, edge_idx);

      //     if let Some(vertex_idx) = mesh.collapse_edge(
      //       edge_idx,
      //       &mut store_modified,
      //       &mut store_removed,
      //     ) {
      //       mesh.set_position(vertex_idx, &avg_pos);
      //     }

      //     removed.extend(store_removed.iter().cloned());
      //   }
      // }

      let mut edge_op = mesh.initial_edge();

      while let Some(edge_idx) = edge_op {
        if let ([l, r, top], Some(bottom)) = mesh.get_edge_neighbors(edge_idx) {
          let l_degree = mesh.degree(l) as i32;
          let r_degree = mesh.degree(r) as i32;
          let top_degree = mesh.degree(top) as i32;
          let bottom_degree = mesh.degree(bottom) as i32;

          let flip_dev = (l_degree - 7).abs()
            + (r_degree - 7).abs()
            + (top_degree - 5).abs()
            + (bottom_degree - 5).abs();
          let no_flip_dev = (l_degree - 6).abs()
            + (r_degree - 6).abs()
            + (top_degree - 6).abs()
            + (bottom_degree - 6).abs();

          if flip_dev < no_flip_dev && l_degree > 3 && r_degree > 3 {
            mesh.flip_edge(edge_idx);
          }
        }

        edge_op = mesh.next_edge(edge_idx);
      }

      let mut vertex_op = mesh.initial_vertex();

      let mut new_positions = Vec::new();
      let mut neighbors = Vec::new();
      let mut store = Vec::new();

      while let Some(vertex_idx) = vertex_op {
        mesh.get_vertex_neighbors(vertex_idx, &mut neighbors);

        let centroid = neighbors
          .iter()
          .fold(Vector3::zeros(), |acc, other_vertex_idx| {
            acc + mesh.get_position(*other_vertex_idx)
          })
          / neighbors.len() as f32;

        let orig_position = mesh.get_position(vertex_idx);

        let diff = centroid - orig_position;

        let normal = mesh.get_vertex_normal(vertex_idx, &mut store);

        let delta = diff - (normal.dot(&diff)) * normal;

        new_positions
          .push((vertex_idx, orig_position + self.smoothing_weight * delta));

        vertex_op = mesh.next_vertex(vertex_idx);
      }

      for (vertex_idx, new_position) in new_positions {
        mesh.set_position(vertex_idx, &new_position);
      }
    }
  }
}
