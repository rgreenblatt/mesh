use crate::data_structure::base::Face;
use crate::data_structure::base::Vertex;
use crate::data_structure::DataStructure;

use std::collections::HashMap;

struct HalfEdgeRef {
  twin_idx: Option<usize>,
  next_idx: usize,
  vertex_idx: usize,
  edge_idx: usize,
  face_idx: usize,
}

struct VertexRef {
  half_edge_idx: usize,
  vertex: Vertex,
}

struct EdgeRef {
  half_edge_idx: usize,
}

struct FaceRef {
  half_edge_idx: usize,
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
    &self.half_edge_refs[half_edge.next_idx]
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
      let face_idx = face_refs.len();
      let next_vertex = [face[1], face[2], face[0]];
      let start_idx = half_edge_refs.len();
      let next_idxs = [start_idx + 1, start_idx + 2, start_idx];
      for ((vertex_orig_idx, next_vertex_orig_idx), next_idx) in
        face.iter().zip(next_vertex.iter()).zip(next_idxs.iter())
      {
        let half_edge_idx = half_edge_refs.len();

        let vertex_idx =
          match vertex_orig_idx_to_vertex_new_idx.get(vertex_orig_idx) {
            Some(v) => *v,
            None => {
              let vertex_idx = vertex_refs.len();
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
            half_edge_refs[twin_idx].twin_idx = Some(half_edge_idx);

            Some(twin_idx)
          }
        };

        let edge_idx = match twin_idx {
          Some(v) => half_edge_refs[v].edge_idx,
          None => {
            edge_refs.push(EdgeRef { half_edge_idx });
            edge_refs.len() - 1
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

  fn to_vecs(&mut self) -> (Vec<Vertex>, Vec<Face>) {
    self.collapse();

    let mut vertices = Vec::new();

    for v in &self.vertex_refs {
      vertices.push(v.vertex);
    }

    let mut faces = Vec::new();

    for v in &self.face_refs {
      let half_edge = &self.half_edge_refs[v.half_edge_idx];
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
