use crate::mesh_operation::Operation;
use crate::DataStructure;
use crate::Vertex;

use clap::Clap;
use nalgebra::base::Matrix4;
use nalgebra::base::Vector4;
use ordered_float::NotNan;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Clap)]
pub struct Simplify {
  faces_to_remove: u32,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct EdgeCost {
  cost: Reverse<NotNan<f32>>,
  edge_idx: usize,
}

fn get_quadric<D: DataStructure>(
  mesh: &D,
  vertex: D::VertexIdx,
  neighbors: &mut Vec<D::VertexIdx>,
) -> Matrix4<f32> {
  let boundary = mesh.get_vertex_neighbors(vertex, neighbors);
  assert!(!boundary);
  assert!(!neighbors.is_empty());

  let mut quadric = Matrix4::zeros();

  let p_0 = mesh.get_position(vertex);

  for i in 0..(neighbors.len() - 1) {
    let p_1 = mesh.get_position(neighbors[i]);
    let p_2 = mesh.get_position(neighbors[i + 1]);

    let normal = ((p_1 - p_0).cross(&(p_2 - p_0))).normalize();

    let d = -p_0.dot(&normal);

    let v = Vector4::new(normal[0], normal[1], normal[2], d);

    quadric += v * v.transpose();
  }

  quadric
}

fn get_best_position_cost<D: DataStructure>(
  mesh: &D,
  vertex_first: D::VertexIdx,
  vertex_second: D::VertexIdx,
  quadric_first: &Matrix4<f32>,
  quadric_second: &Matrix4<f32>,
) -> (Vertex, NotNan<f32>) {
  (Vertex::zeros(), NotNan::new(0.0).unwrap())
}

impl Operation for Simplify {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    let mut edge_heap = BinaryHeap::new();
    // let mut consumed = HashMap::new();

    // bad storage/recomputation, but might not matter
    let mut edge_info = Vec::new();

    let mut edge_op = mesh.initial_edge();

    let mut neighbors = Vec::new();

    // let mut invalid_edges = HashMap::new();

    while let Some(edge) = edge_op {
      let [first, second] = mesh.get_endpoints(edge);

      let first_quadric = get_quadric(mesh, first, &mut neighbors);
      let second_quadric = get_quadric(mesh, second, &mut neighbors);

      let edge_idx = edge_info.len();

      let (best_position, cost) = get_best_position_cost(
        mesh,
        first,
        second,
        &first_quadric,
        &second_quadric,
      );

      edge_info.push((first_quadric, second_quadric, best_position, edge));

      edge_heap.push(EdgeCost {
        cost: Reverse(cost),
        edge_idx,
      });
      edge_op = mesh.next_edge(edge);
    }

    let initial_num_faces = mesh.num_faces();

    assert!(initial_num_faces < self.faces_to_remove as usize);

    let final_num_faces = initial_num_faces - self.faces_to_remove as usize;

    while mesh.num_faces() >= final_num_faces {
      let EdgeCost { cost, edge_idx } =
        edge_heap.pop().expect("Heap shouldn't be empty");

      let (first_quadric, second_quadric, best_position, edge) =
        edge_info[edge_idx];

      // if invalid_edges.contains(&edge) {
      //   continue;
      // }

      let (
        new_vertex,
        [update_edge_0, update_edge_1, invalid_edge_0, invalid_edge_1],
      ) = mesh.collapse_edge(edge);
    }

    // let (new, _) = mesh.split_edge(mesh.initial_edge().unwrap());
    // mesh.set_position(new, &Vertex::new(1.0, 1.0, 1.0));
  }
}
