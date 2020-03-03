use crate::mesh_operation::Operation;
use crate::DataStructure;
use crate::IndexType;
use crate::Vector3;
use crate::get_normal;

use clap::Clap;
use nalgebra::base::{dimension::U1, Matrix4, Vector4};
use ordered_float::NotNan;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clap)]
pub struct Simplify {
  faces_to_remove: u32,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct EdgeCost {
  cost: Reverse<NotNan<f32>>,
  edge_idx: IndexType,
  count: u8,
}

fn get_quadric<D: DataStructure>(
  mesh: &D,
  face_idx: IndexType,
) -> Matrix4<f32> {
  let [v_0, v_1, v_2] = mesh.get_face_neighbors(face_idx);

  let p_0 = mesh.get_position(v_0);
  let p_1 = mesh.get_position(v_1);
  let p_2 = mesh.get_position(v_2);

  let normal = get_normal([p_0, p_1, p_2]);

  let d = -p_0.dot(&normal);

  debug_assert!((d - (-p_1.dot(&normal))) < 1e-4);
  debug_assert!((d - (-p_2.dot(&normal))) < 1e-4);

  let v = Vector4::new(normal[0], normal[1], normal[2], d);

  v * v.transpose()
}

fn get_best_position_cost<D: DataStructure>(
  mesh: &D,
  vertex_first: IndexType,
  vertex_second: IndexType,
  quadric_first: &Matrix4<f32>,
  quadric_second: &Matrix4<f32>,
) -> (Vector3, NotNan<f32>) {
  let mut combined_quadric = quadric_first + quadric_second;

  combined_quadric.row_mut(3)[0] = 0.0;
  combined_quadric.row_mut(3)[1] = 0.0;
  combined_quadric.row_mut(3)[2] = 0.0;
  combined_quadric.row_mut(3)[3] = 1.0;

  let optimal_position = combined_quadric
    .try_inverse()
    .map(|v| (v * Vector4::new(0.0, 0.0, 0.0, 1.0)).remove_fixed_rows::<U1>(3))
    .unwrap_or_else(|| {
      dbg!("Not invertible!");
      0.5 * (mesh.get_position(vertex_first) + mesh.get_position(vertex_second))
    });

  let optimal_position_4 = Vector4::new(
    optimal_position[0],
    optimal_position[1],
    optimal_position[2],
    1.0,
  );

  let cost =
    (optimal_position_4.transpose() * combined_quadric * optimal_position_4)[0];

  if cfg!(debug_assertions) {
    let mut neighbors = Vec::new();

    mesh.get_vertex_neighbors(vertex_first, &mut neighbors);
    debug_assert!(neighbors.contains(&vertex_second));
    mesh.get_vertex_neighbors(vertex_second, &mut neighbors);
    debug_assert!(neighbors.contains(&vertex_first));
  }

  if cost <= 0.0 {
    dbg!(optimal_position_4);
    dbg!(combined_quadric);
    dbg!(vertex_first);
    dbg!(vertex_second);

    dbg!(mesh.get_position(vertex_first));
    dbg!(mesh.get_position(vertex_second));

    dbg!(cost);
    
    dbg!("oh no!");
  }

  // debug_assert!(cost > 0.0);

  (optimal_position, NotNan::new(cost).unwrap())
}

impl Operation for Simplify {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    let mut face_quadrics = Vec::new();
    face_quadrics.resize(mesh.max_idx_faces(), None);
    let mut vertex_quadrics = Vec::new();
    vertex_quadrics.resize(mesh.max_idx_vertices(), None);

    let mut adjacent_faces = Vec::new();

    let mut edge_op = mesh.initial_edge();

    let mut get_face_quadric = |face_idx| {
      if let Some(quadric) = face_quadrics[face_idx as usize] {
        quadric
      } else {
        let quadric = get_quadric(mesh, face_idx);

        face_quadrics[face_idx as usize] = Some(quadric);

        quadric
      }
    };

    let mut get_vertex_quadric = |vertex_idx| {
      if let Some(quadric) = vertex_quadrics[vertex_idx as usize] {
        quadric
      } else {
        mesh.get_vertex_adjacent_faces(vertex_idx, &mut adjacent_faces);

        let quadric =
          adjacent_faces
            .iter()
            .fold(Matrix4::zeros(), |acc, face_idx| {
              if vertex_idx == 869 || vertex_idx == 946 {
                dbg!(face_idx);
              }
              acc + get_face_quadric(*face_idx)
            });

        vertex_quadrics[vertex_idx as usize] = Some(quadric);

        quadric
      }
    };

    let mut edge_heap = BinaryHeap::new();
    let mut edge_info = Vec::new();

    edge_info.resize(mesh.max_idx_edges(), None);

    while let Some(edge_idx) = edge_op {
      let [first, second] = mesh.get_endpoints(edge_idx);

      let first_quadric = get_vertex_quadric(first);
      let second_quadric = get_vertex_quadric(second);

      let (best_position, cost) = get_best_position_cost(
        mesh,
        first,
        second,
        &first_quadric,
        &second_quadric,
      );

      edge_heap.push(EdgeCost {
        cost: Reverse(cost),
        edge_idx,
        count: 0,
      });

      edge_info[edge_idx as usize] = Some((best_position, first, second, 0));

      edge_op = mesh.next_edge(edge_idx);
    }

    let initial_num_faces = dbg!(mesh.num_faces());
    dbg!(self.faces_to_remove);

    assert!((self.faces_to_remove as usize) < initial_num_faces);

    let final_num_faces = initial_num_faces - self.faces_to_remove as usize;

    while mesh.num_faces() >= final_num_faces {
      let EdgeCost {
        edge_idx,
        count,
        cost,
      } = edge_heap.pop().expect("Heap shouldn't be empty");

      // TODO: when will this occur
      if edge_info[edge_idx as usize].is_none() {
        continue;
      }

      let (best_position, first_vertex_idx, second_vertex_idx, true_count) =
        edge_info[edge_idx as usize].unwrap();

      if count < true_count {
        continue;
      }

      assert_eq!(true_count, count);

      dbg!(edge_idx);
      dbg!(count);
      dbg!(cost);

      if let Some((
        [new_vertex, edge_0_vertex, edge_1_vertex],
        [update_edge_0, update_edge_1, invalid_edge_0, invalid_edge_1],
      )) = mesh.collapse_edge(edge_idx)
      {
        dbg!(update_edge_0);
        dbg!(update_edge_1);
        dbg!(invalid_edge_0);
        dbg!(invalid_edge_1);

        mesh.set_position(new_vertex, &best_position);

        edge_info[invalid_edge_0 as usize] = None;
        edge_info[invalid_edge_1 as usize] = None;

        let new_vertex_quadric = vertex_quadrics[first_vertex_idx as usize]
          .unwrap()
          + vertex_quadrics[second_vertex_idx as usize].unwrap();

        vertex_quadrics[new_vertex as usize] = Some(new_vertex_quadric);

        let mut update = |vertex_idx, edge_idx| {
          let (best_position, cost) = get_best_position_cost(
            mesh,
            new_vertex,
            vertex_idx, // verify order unimportant
            &new_vertex_quadric,
            &vertex_quadrics[vertex_idx as usize].unwrap(),
          );

          let mut old_count = 0;

          if let Some((_, _, _, count)) = edge_info[edge_idx as usize] {
            old_count = count;
          }

          let count = old_count + 1;

          edge_info[edge_idx as usize] =
            Some((best_position, new_vertex, vertex_idx, count));

          edge_heap.push(EdgeCost {
            edge_idx,
            cost: Reverse(cost),
            count,
          });
        };

        update(edge_0_vertex, update_edge_0);
        update(edge_1_vertex, update_edge_1);
      }
    }
  }
}
