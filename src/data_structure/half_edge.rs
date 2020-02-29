use crate::data_structure::base::Face;
use crate::data_structure::base::Vertex;
use crate::data_structure::DataStructure;

use std::collections::HashMap;

type IndexType = u32;

struct HalfEdgeRef {
  twin_idx: Option<IndexType>,
  next_idx: IndexType,
  vertex_idx: IndexType,
  edge_idx: IndexType,
  face_idx: IndexType,
}

struct VertexRef {
  half_edge_idx: IndexType,
  vertex: Vertex,
}

struct EdgeRef {
  half_edge_idx: IndexType,
}

struct FaceRef {
  half_edge_idx: IndexType,
  // Normal?
}

pub struct HalfEdge {
  half_edge_refs: Vec<HalfEdgeRef>,
  vertex_refs: Vec<VertexRef>,
  edge_refs: Vec<EdgeRef>,
  face_refs: Vec<FaceRef>,
}

impl HalfEdge {
  fn collapse(&mut self) {}

  fn get_next(&self, half_edge: &HalfEdgeRef) -> &HalfEdgeRef {
    &self.half_edge_refs[half_edge.next_idx as usize]
  }
}

impl DataStructure for HalfEdge {
  fn from_iters<IterVert, IterFace>(vertices: IterVert, faces: IterFace) -> Self
  where
    IterVert: IntoIterator<Item = Vertex>,
    IterFace: IntoIterator<Item = Face>,
  {
    let vertices_vec: Vec<Vertex> = vertices.into_iter().collect();

    let mut vertex_orig_idx_to_vertex_new_idx = HashMap::new();
    let mut vertex_pair_to_half_edge_idx = HashMap::new();

    let mut half_edge_refs = Vec::<HalfEdgeRef>::new();
    let mut vertex_refs = Vec::new();
    let mut edge_refs = Vec::new();
    let mut face_refs = Vec::new();

    for face in faces.into_iter() {
      let face_idx = face_refs.len() as IndexType;
      let next_vertex = [face[1], face[2], face[0]];
      let start_idx = half_edge_refs.len() as IndexType;
      let next_idxs = [start_idx + 1, start_idx + 2, start_idx];
      for ((vertex_orig_idx, next_vertex_orig_idx), next_idx) in
        face.iter().zip(next_vertex.iter()).zip(next_idxs.iter())
      {
        let half_edge_idx = half_edge_refs.len() as IndexType;

        let vertex_idx =
          match vertex_orig_idx_to_vertex_new_idx.get(vertex_orig_idx) {
            Some(v) => *v,
            None => {
              let vertex_idx = vertex_refs.len() as IndexType;
              vertex_orig_idx_to_vertex_new_idx
                .insert(*vertex_orig_idx, vertex_idx);
              vertex_refs.push(VertexRef {
                half_edge_idx,
                vertex: vertices_vec[*vertex_orig_idx as usize],
              });
              vertex_idx
            }
          };

        let twin_idx = match vertex_pair_to_half_edge_idx
          .get(&(*vertex_orig_idx, *next_vertex_orig_idx))
        {
          None => {
            vertex_pair_to_half_edge_idx
              .insert((*next_vertex_orig_idx, *vertex_orig_idx), half_edge_idx);
            None
          }
          Some(v) => {
            let twin_idx = *v;
            half_edge_refs[twin_idx as usize].twin_idx = Some(half_edge_idx);

            Some(twin_idx)
          }
        };

        let edge_idx = match twin_idx {
          Some(v) => half_edge_refs[v as usize].edge_idx,
          None => {
            edge_refs.push(EdgeRef { half_edge_idx });
            (edge_refs.len() - 1) as IndexType
          }
        };

        half_edge_refs.push(HalfEdgeRef {
          twin_idx,
          next_idx: *next_idx,
          vertex_idx,
          edge_idx,
          face_idx,
        })
      }

      face_refs.push(FaceRef {
        half_edge_idx: start_idx,
      });
    }

    HalfEdge {
      half_edge_refs,
      vertex_refs,
      edge_refs,
      face_refs,
    }
  }

  type EdgeKey = IndexType;
  type VertexKey = IndexType;

  fn initial_edge(&self) -> Self::EdgeKey {
    // TODO: filter out invalid edges
    0
  }

  fn next_edge(&self, key: &Self::EdgeKey) -> Self::EdgeKey {
    // TODO: filter out invalid edges
    *key + 1
  }

  fn flip_edge(&mut self, key: &Self::EdgeKey) {
    let edge = &self.edge_refs[*key as usize];

    let first_half_edge = &self.half_edge_refs[edge.half_edge_idx as usize];

    if let Some(twin_idx) = first_half_edge.twin_idx {
      let second_half_edge = &self.half_edge_refs[twin_idx as usize];

      let first_next_half_edge_idx = first_half_edge.next_idx;
      let second_next_half_edge_idx = second_half_edge.next_idx;

      let first_next_half_edge = self.get_next(first_half_edge);
      let second_next_half_edge = self.get_next(second_half_edge);

      let new_vertex_first = self.get_next(second_next_half_edge).vertex_idx;
      let new_vertex_second = self.get_next(first_next_half_edge).vertex_idx;

      let new_face_next_first = second_half_edge.face_idx;
      let new_face_next_second = first_half_edge.face_idx;

      // switch faces of invalidated half_edges
      self.half_edge_refs[first_next_half_edge_idx as usize].face_idx =
        new_face_next_first;
      self.half_edge_refs[second_next_half_edge_idx as usize].face_idx =
        new_face_next_second;

      self.half_edge_refs[edge.half_edge_idx as usize].vertex_idx =
        new_vertex_first;
      self.half_edge_refs[twin_idx as usize].vertex_idx = new_vertex_second;
    } else {
      // later consider making this do nothing
      panic!("AHHHH boundary");
    }
  }

  fn to_vecs(&mut self) -> (Vec<Vertex>, Vec<Face>) {
    self.collapse();

    let mut vertices = Vec::new();

    for v in &self.vertex_refs {
      vertices.push(v.vertex);
    }

    let mut faces = Vec::new();

    for v in &self.face_refs {
      let half_edge = &self.half_edge_refs[v.half_edge_idx as usize];
      let next_half_edge = self.get_next(half_edge);
      let next_next_half_edge = self.get_next(next_half_edge);
      faces.push([
        half_edge.vertex_idx as u32,
        next_half_edge.vertex_idx as u32,
        next_next_half_edge.vertex_idx as u32,
      ]);
    }

    (vertices, faces)
  }
}
