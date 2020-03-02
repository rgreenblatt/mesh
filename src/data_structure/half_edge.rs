use crate::data_structure::base::Face;
use crate::data_structure::base::IndexType;
use crate::data_structure::base::Vertex;
use crate::data_structure::DataStructure;

use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::iter::FromIterator;

struct HalfEdgeRef {
  twin_idx: Option<IndexType>,
  next_idx: IndexType,
  vertex_idx: IndexType,
  edge_idx: IndexType,
  face_idx: IndexType,
}

#[derive(Clone)]
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
  vertex_refs: Vec<Option<VertexRef>>,
  edge_refs: Vec<Option<EdgeRef>>,
  face_refs: Vec<Option<FaceRef>>,
  num_removed_vertices: usize,
  num_removed_edges: usize,
  num_removed_faces: usize,
}

enum Offset {
  Current,
  Next,
  NextNext,
}

impl HalfEdge {
  fn collapse(&mut self) {}

  fn get_next(&self, half_edge: &HalfEdgeRef) -> &HalfEdgeRef {
    &self.half_edge_refs[half_edge.next_idx as usize]
  }

  fn relative_get(&self, idx: IndexType, offset: Offset) -> &HalfEdgeRef {
    let first = &self.half_edge_refs[idx as usize];

    match offset {
      Offset::Current => first,
      Offset::Next => &self.half_edge_refs[first.next_idx as usize],
      Offset::NextNext => {
        &self.half_edge_refs
          [self.half_edge_refs[first.next_idx as usize].next_idx as usize]
      }
    }
  }

  fn verify_half_edge_valid(&self, half_edge_idx: IndexType) {
    if cfg!(debug_assertions) {
      let half_edge = &self.half_edge_refs[half_edge_idx as usize];
      debug_assert_eq!(
        self.relative_get(half_edge_idx, Offset::Next).vertex_idx,
        self.half_edge_refs[half_edge.twin_idx.unwrap() as usize].vertex_idx
      );

      debug_assert_eq!(
        self.relative_get(half_edge_idx, Offset::NextNext).next_idx,
        half_edge_idx
      );
    }
  }

  fn verify_vertex_valid(&self, vertex: IndexType) {
    if cfg!(debug_assertions) {
      let mut neighbors = Vec::new();
      self.get_vertex_neighbors(vertex, &mut neighbors);
      debug_assert!(!neighbors.contains(&vertex));
      self.verify_half_edge_valid(
        self.vertex_refs[vertex as usize]
          .as_ref()
          .unwrap()
          .half_edge_idx,
      );
      for idx in neighbors {
        self.verify_half_edge_valid(
          self.vertex_refs[idx as usize]
            .as_ref()
            .unwrap()
            .half_edge_idx,
        )
      }
    }
  }

  fn verify_edge_valid(&self, edge_idx: IndexType) {
    if cfg!(debug_assertions) {
      let [left, right] = self.get_endpoints(edge_idx);
      if edge_idx == 25 {
        dbg!(left);
        dbg!(right);
      }
      debug_assert_ne!(left, right);
      self.verify_vertex_valid(left);
      self.verify_vertex_valid(right);
    }
  }

  fn get_at<T>(start: IndexType, vals: &[Option<T>]) -> Option<IndexType> {
    for check in start..(vals.len() as IndexType) {
      if vals[check as usize].is_some() {
        return Some(check);
      }
    }

    None
  }

  fn get_face_ref_neighbors(&self, face: &FaceRef) -> [IndexType; 3] {
    let half_edge = &self.half_edge_refs[face.half_edge_idx as usize];
    let next_half_edge = self.get_next(half_edge);
    let next_next_half_edge = self.get_next(next_half_edge);
    [
      half_edge.vertex_idx,
      next_half_edge.vertex_idx,
      next_next_half_edge.vertex_idx,
    ]
  }

  fn get_start_iter_half_edge_idx(
    &self,
    vertex_idx: IndexType,
  ) -> (bool, IndexType) {
    let half_edge_idx_orig = self.vertex_refs[vertex_idx as usize]
      .as_ref()
      .unwrap()
      .half_edge_idx;
    let mut half_edge_idx = half_edge_idx_orig;

    let mut first = true;
    let mut has_boundary = false;

    // iterate until we hit one side
    while first || half_edge_idx != half_edge_idx_orig {
      let next_edge = self.relative_get(half_edge_idx, Offset::NextNext);
      if let Some(new_half_edge_idx) = next_edge.twin_idx {
        half_edge_idx = new_half_edge_idx;
      } else {
        has_boundary = true;
        break;
      }

      first = false;
    }

    (has_boundary, half_edge_idx)
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
              vertex_refs.push(Some(VertexRef {
                half_edge_idx,
                vertex: vertices_vec[*vertex_orig_idx as usize],
              }));
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
            edge_refs.push(Some(EdgeRef { half_edge_idx }));
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

      face_refs.push(Some(FaceRef {
        half_edge_idx: start_idx,
      }));
    }

    let vertex_length = vertex_refs.len() as IndexType;
    let half_edge_length = half_edge_refs.len() as IndexType;

    let out = HalfEdge {
      half_edge_refs,
      vertex_refs,
      edge_refs,
      face_refs,
      num_removed_vertices: 0,
      num_removed_edges: 0,
      num_removed_faces: 0,
    };

    for idx in 0..vertex_length {
      out.verify_vertex_valid(idx);
    }

    for idx in 0..half_edge_length {
      out.verify_half_edge_valid(idx);
    }

    out
  }

  fn max_idx_vertices(&self) -> usize {
    self.vertex_refs.len()
  }

  fn max_idx_edges(&self) -> usize {
    self.edge_refs.len()
  }

  fn max_idx_faces(&self) -> usize {
    self.face_refs.len()
  }

  fn num_vertices(&self) -> usize {
    self.vertex_refs.len() - self.num_removed_vertices
  }

  fn num_edges(&self) -> usize {
    self.edge_refs.len() - self.num_removed_edges
  }

  fn num_faces(&self) -> usize {
    self.face_refs.len() - self.num_removed_faces
  }

  fn initial_vertex(&self) -> Option<IndexType> {
    HalfEdge::get_at(0, &self.vertex_refs)
  }

  fn next_vertex(&self, key: IndexType) -> Option<IndexType> {
    HalfEdge::get_at(key + 1, &self.vertex_refs)
  }

  fn initial_edge(&self) -> Option<IndexType> {
    HalfEdge::get_at(0, &self.edge_refs)
  }

  fn next_edge(&self, key: IndexType) -> Option<IndexType> {
    HalfEdge::get_at(key + 1, &self.edge_refs)
  }

  fn initial_face(&self) -> Option<IndexType> {
    HalfEdge::get_at(0, &self.face_refs)
  }

  fn next_face(&self, key: IndexType) -> Option<IndexType> {
    HalfEdge::get_at(key + 1, &self.face_refs)
  }

  fn flip_edge(&mut self, key: IndexType) {
    // see page 24 of lecture slides "meshes_geoprocessing" for
    // a,b,c,d reference

    let edge = &self.edge_refs[key as usize].as_ref().unwrap();

    let b_c_idx = edge.half_edge_idx;
    let b_c_half_edge = self.relative_get(b_c_idx, Offset::Current);

    if let Some(c_b_idx) = b_c_half_edge.twin_idx {
      let c_b_half_edge = &self.half_edge_refs[c_b_idx as usize];

      let a_d_idx = c_b_idx;
      let d_a_idx = b_c_idx;

      let c_a_idx = self.relative_get(b_c_idx, Offset::Current).next_idx;
      let a_b_idx = self.relative_get(b_c_idx, Offset::Next).next_idx;

      let b_d_idx = self.relative_get(c_b_idx, Offset::Current).next_idx;
      let d_c_idx = self.relative_get(c_b_idx, Offset::Next).next_idx;

      self.verify_half_edge_valid(c_a_idx);
      self.verify_half_edge_valid(b_d_idx);
      self.verify_half_edge_valid(b_c_idx);
      self.verify_half_edge_valid(c_b_idx);

      let c_vertex_idx = self.relative_get(b_c_idx, Offset::Next).vertex_idx;
      let a_vertex_idx =
        self.relative_get(b_c_idx, Offset::NextNext).vertex_idx;
      let b_vertex_idx = self.relative_get(c_b_idx, Offset::Next).vertex_idx;
      let d_vertex_idx =
        self.relative_get(c_b_idx, Offset::NextNext).vertex_idx;

      let c_a_d_face = c_b_half_edge.face_idx;
      let b_a_d_face = b_c_half_edge.face_idx;

      // switch faces of invalidated half_edges
      self.half_edge_refs[c_a_idx as usize].face_idx = c_a_d_face;
      self.half_edge_refs[b_d_idx as usize].face_idx = b_a_d_face;

      self.half_edge_refs[c_a_idx as usize].next_idx = a_d_idx;
      self.half_edge_refs[b_d_idx as usize].next_idx = d_a_idx;

      self.half_edge_refs[a_d_idx as usize].vertex_idx = a_vertex_idx;
      self.half_edge_refs[d_a_idx as usize].vertex_idx = d_vertex_idx;

      self.half_edge_refs[a_d_idx as usize].next_idx = d_c_idx;
      self.half_edge_refs[d_a_idx as usize].next_idx = a_b_idx;

      self.half_edge_refs[d_c_idx as usize].next_idx = c_a_idx;
      self.half_edge_refs[a_b_idx as usize].next_idx = b_d_idx;

      self.face_refs[c_a_d_face as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = c_a_idx;
      self.face_refs[b_a_d_face as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = b_d_idx;

      self.vertex_refs[c_vertex_idx as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = c_a_idx;
      self.vertex_refs[b_vertex_idx as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = b_d_idx;

      self.verify_half_edge_valid(c_a_idx);
      self.verify_half_edge_valid(b_d_idx);
      self.verify_half_edge_valid(b_c_idx);
      self.verify_half_edge_valid(c_b_idx);

      self.verify_vertex_valid(a_vertex_idx);
      self.verify_vertex_valid(b_vertex_idx);
      self.verify_vertex_valid(c_vertex_idx);
      self.verify_vertex_valid(d_vertex_idx);
    } else {
      // later consider making this do nothing
      panic!("AHHHH boundary");
    }
  }

  fn split_edge(&mut self, key: IndexType) -> (IndexType, [IndexType; 4]) {
    // see page 26 of lecture slides "meshes_geoprocessing" for
    // a,b,c,d,m reference

    let edge = &self.edge_refs[key as usize].as_ref().unwrap();

    // b->c will become m->c
    let b_c_idx = edge.half_edge_idx;

    let twin_idx = self.relative_get(b_c_idx, Offset::Current).twin_idx;

    // twin/second: c->b will become c->m
    if let Some(c_b_idx) = twin_idx {
      // HALF EDGES:

      let m_c_idx = b_c_idx;
      let c_m_idx = c_b_idx;

      let c_a_idx = self.relative_get(b_c_idx, Offset::Current).next_idx;
      let a_b_idx = self.relative_get(b_c_idx, Offset::Next).next_idx;

      let b_d_idx = self.relative_get(c_b_idx, Offset::Current).next_idx;
      let d_c_idx = self.relative_get(c_b_idx, Offset::Next).next_idx;

      // Order of new half edges is:
      //  - m->d
      //  - d->m
      //  - m->b
      //  - b->m
      //  - m->a
      //  - a->m
      let m_d_idx = self.half_edge_refs.len() as IndexType;
      let d_m_idx = (self.half_edge_refs.len() + 1) as IndexType;
      let m_b_idx = (self.half_edge_refs.len() + 2) as IndexType;
      let b_m_idx = (self.half_edge_refs.len() + 3) as IndexType;
      let m_a_idx = (self.half_edge_refs.len() + 4) as IndexType;
      let a_m_idx = (self.half_edge_refs.len() + 5) as IndexType;

      // VERTICES:

      let a_vertex_idx = self.relative_get(a_b_idx, Offset::Current).vertex_idx;
      let b_vertex_idx = self.relative_get(b_d_idx, Offset::Current).vertex_idx;
      let c_vertex_idx = self.relative_get(c_b_idx, Offset::Current).vertex_idx;
      let d_vertex_idx = self.relative_get(d_c_idx, Offset::Current).vertex_idx;

      self.verify_half_edge_valid(c_a_idx);
      self.verify_half_edge_valid(a_b_idx);
      self.verify_half_edge_valid(b_d_idx);
      self.verify_half_edge_valid(d_c_idx);
      self.verify_half_edge_valid(b_c_idx);
      self.verify_half_edge_valid(c_b_idx);

      self.verify_vertex_valid(a_vertex_idx);
      self.verify_vertex_valid(b_vertex_idx);
      self.verify_vertex_valid(c_vertex_idx);
      self.verify_vertex_valid(d_vertex_idx);

      // ensure still valid
      self.vertex_refs[b_vertex_idx as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = b_d_idx;
      self.vertex_refs[c_vertex_idx as usize]
        .as_mut()
        .unwrap()
        .half_edge_idx = c_a_idx;

      // new vertex (m)
      let m_vertex_idx = self.vertex_refs.len() as IndexType;

      // add vertex m (for now copy of b)
      self.vertex_refs.push(Some(VertexRef {
        half_edge_idx: m_c_idx,
        vertex: self.vertex_refs[b_vertex_idx as usize]
          .as_ref()
          .unwrap()
          .vertex,
      }));

      // EDGES:

      // Order of new edges is:
      //  - m<->d
      //  - m<->b
      //  - m<->a
      let m_c_edge_idx = key;
      let m_d_edge_idx = self.edge_refs.len() as IndexType;
      let m_b_edge_idx = (self.edge_refs.len() + 1) as IndexType;
      let m_a_edge_idx = (self.edge_refs.len() + 2) as IndexType;

      // m_d
      self.edge_refs.push(Some(EdgeRef {
        half_edge_idx: m_d_idx,
      }));
      // m_b
      self.edge_refs.push(Some(EdgeRef {
        half_edge_idx: m_b_idx,
      }));
      // m_a
      self.edge_refs.push(Some(EdgeRef {
        half_edge_idx: m_a_idx,
      }));

      // FACES:

      // bdc becomes mdc
      let m_d_c_face_idx = self.half_edge_refs[c_m_idx as usize].face_idx;
      // cab becomes mca
      let m_c_a_face_idx = self.half_edge_refs[m_c_idx as usize].face_idx;

      // Order of new faces is:
      //  - mab
      //  - mbd
      let m_a_b_face_idx = self.face_refs.len() as IndexType;
      let m_b_d_face_idx = (self.face_refs.len() + 1) as IndexType;

      self.face_refs[m_d_c_face_idx as usize] = Some(FaceRef {
        half_edge_idx: c_m_idx,
      });
      self.face_refs[m_c_a_face_idx as usize] = Some(FaceRef {
        half_edge_idx: m_c_idx,
      });
      // mab
      self.face_refs.push(Some(FaceRef {
        half_edge_idx: m_a_idx,
      }));
      // mbd
      self.face_refs.push(Some(FaceRef {
        half_edge_idx: m_b_idx,
      }));

      self.half_edge_refs[m_c_idx as usize].vertex_idx = m_vertex_idx;
      self.half_edge_refs[c_m_idx as usize].next_idx = m_d_idx;

      self.half_edge_refs[b_d_idx as usize].face_idx = m_b_d_face_idx;
      self.half_edge_refs[b_d_idx as usize].next_idx = d_m_idx;
      self.half_edge_refs[a_b_idx as usize].face_idx = m_a_b_face_idx;
      self.half_edge_refs[a_b_idx as usize].next_idx = b_m_idx;

      self.half_edge_refs[d_c_idx as usize].next_idx = c_m_idx;
      self.half_edge_refs[c_a_idx as usize].next_idx = a_m_idx;

      // m -> d
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(d_m_idx),
        next_idx: d_c_idx,
        vertex_idx: m_vertex_idx,
        edge_idx: m_d_edge_idx,
        face_idx: m_d_c_face_idx,
      });

      // d -> m
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(m_d_idx),
        next_idx: m_b_idx,
        vertex_idx: d_vertex_idx,
        edge_idx: m_d_edge_idx,
        face_idx: m_b_d_face_idx,
      });

      // m -> b
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(b_m_idx),
        next_idx: b_d_idx,
        vertex_idx: m_vertex_idx,
        edge_idx: m_b_edge_idx,
        face_idx: m_b_d_face_idx,
      });

      // b -> m
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(m_b_idx),
        next_idx: m_a_idx,
        vertex_idx: b_vertex_idx,
        edge_idx: m_b_edge_idx,
        face_idx: m_a_b_face_idx,
      });

      // m -> a
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(a_m_idx),
        next_idx: a_b_idx,
        vertex_idx: m_vertex_idx,
        edge_idx: m_a_edge_idx,
        face_idx: m_a_b_face_idx,
      });

      // a -> m
      self.half_edge_refs.push(HalfEdgeRef {
        twin_idx: Some(m_a_idx),
        next_idx: m_c_idx,
        vertex_idx: a_vertex_idx,
        edge_idx: m_a_edge_idx,
        face_idx: m_c_a_face_idx,
      });

      self.verify_half_edge_valid(c_a_idx);
      self.verify_half_edge_valid(a_b_idx);
      self.verify_half_edge_valid(b_d_idx);
      self.verify_half_edge_valid(d_c_idx);
      self.verify_half_edge_valid(b_c_idx);
      self.verify_half_edge_valid(c_b_idx);
      self.verify_half_edge_valid(m_d_idx);
      self.verify_half_edge_valid(d_m_idx);
      self.verify_half_edge_valid(m_b_idx);
      self.verify_half_edge_valid(b_m_idx);
      self.verify_half_edge_valid(m_a_idx);
      self.verify_half_edge_valid(a_m_idx);

      self.verify_vertex_valid(a_vertex_idx);
      self.verify_vertex_valid(b_vertex_idx);
      self.verify_vertex_valid(c_vertex_idx);
      self.verify_vertex_valid(d_vertex_idx);
      self.verify_vertex_valid(m_vertex_idx);

      (
        m_vertex_idx,
        [m_a_edge_idx, m_d_edge_idx, m_c_edge_idx, m_b_edge_idx],
      )
    } else {
      // later consider making this just split on one side
      panic!("AHHHH boundary");
    }
  }

  fn collapse_edge(
    &mut self,
    key: IndexType,
  ) -> ([IndexType; 3], [IndexType; 4]) {
    // see page 28 of lecture slides "meshes_geoprocessing" for
    // a,b,c,d,m reference

    // remove d, d<->a, d<->b

    let edge = &self.edge_refs[key as usize].as_ref().unwrap();

    // c->d will be removed
    let c_d_idx = edge.half_edge_idx;

    let twin_idx = self.relative_get(c_d_idx, Offset::Current).twin_idx;

    if let Some(d_c_idx) = twin_idx {
      // remove d->a
      let d_a_idx = self.relative_get(c_d_idx, Offset::Current).next_idx;
      // remove a->c
      let a_c_idx = self.relative_get(c_d_idx, Offset::Next).next_idx;

      // remove c->b
      let c_b_idx = self.relative_get(d_c_idx, Offset::Current).next_idx;
      // remove b->d
      let b_d_idx = self.relative_get(d_c_idx, Offset::Next).next_idx;

      // will be removed
      let c_vertex_idx = self.relative_get(c_d_idx, Offset::Current).vertex_idx;
      // will become m
      let d_vertex_idx = self.relative_get(d_c_idx, Offset::Current).vertex_idx;
      let a_vertex_idx = self.relative_get(a_c_idx, Offset::Current).vertex_idx;
      let b_vertex_idx = self.relative_get(b_d_idx, Offset::Current).vertex_idx;

      debug_assert_ne!(a_vertex_idx, c_vertex_idx);
      debug_assert_ne!(b_vertex_idx, c_vertex_idx);
      debug_assert_ne!(d_vertex_idx, c_vertex_idx);
      debug_assert_ne!(a_vertex_idx, d_vertex_idx);
      debug_assert_ne!(b_vertex_idx, d_vertex_idx);
      debug_assert_ne!(a_vertex_idx, b_vertex_idx);

      if cfg!(debug_assertions) {
        let mut c_neighbors = Vec::new();
        self.get_vertex_neighbors(c_vertex_idx, &mut c_neighbors);
        let mut d_neighbors = Vec::new();
        self.get_vertex_neighbors(d_vertex_idx, &mut d_neighbors);

        let first_set = HashSet::<u32>::from_iter(c_neighbors.into_iter());
        let second_set = HashSet::<u32>::from_iter(d_neighbors.into_iter());

        for vertex_idx in first_set.intersection(&second_set) {
          assert!(self.degree(*vertex_idx) > 3);
        }
      }

      let m_vertex_idx = d_vertex_idx;

      {
        let (_, half_edge_idx) = self.get_start_iter_half_edge_idx(c_vertex_idx);

        let mut half_edge_idx_in =
          self.relative_get(half_edge_idx, Offset::Next).next_idx;
        let half_edge_idx_orig_in = half_edge_idx_in;

        let mut first = true;

        while first || half_edge_idx_in != half_edge_idx_orig_in {
          let this_edge_idx = self
            .relative_get(half_edge_idx_in, Offset::Current)
            .next_idx;

          debug_assert_eq!(
            self.relative_get(this_edge_idx, Offset::Current).vertex_idx,
            c_vertex_idx
          );

          self.half_edge_refs[this_edge_idx as usize].vertex_idx = m_vertex_idx;

          if let Some(new_half_edge_idx_in) =
            self.relative_get(half_edge_idx_in, Offset::Next).twin_idx
          {
            half_edge_idx_in = new_half_edge_idx_in;
          } else {
            panic!("boundary!!!");
            // break;
          }

          first = false;
        }
      }

      dbg!(c_vertex_idx);

      self.vertex_refs[c_vertex_idx as usize] = None;

      self.num_removed_vertices += 1;

      // TODO
      let m_b_idx = self.relative_get(b_d_idx, Offset::Current).twin_idx;

      self.vertex_refs[m_vertex_idx as usize] =
        m_b_idx.map(|half_edge_idx| VertexRef {
          vertex: self.vertex_refs[m_vertex_idx as usize]
            .as_ref()
            .unwrap()
            .vertex,
          half_edge_idx,
        });

      // d<->a retained
      let d_a_edge_idx = self.relative_get(d_a_idx, Offset::Current).edge_idx;
      // b<->d retained
      let b_d_edge_idx = self.relative_get(b_d_idx, Offset::Current).edge_idx;
      let m_a_edge_idx = d_a_edge_idx;
      let b_m_edge_idx = b_d_edge_idx;

      // c<->a removed
      let c_a_edge_idx = self.relative_get(a_c_idx, Offset::Current).edge_idx;
      // b<->c removed
      let b_c_edge_idx = self.relative_get(c_b_idx, Offset::Current).edge_idx;

      self.edge_refs[c_a_edge_idx as usize] = None;
      self.edge_refs[b_c_edge_idx as usize] = None;

      self.num_removed_edges += 2;

      // faces are removed
      let c_a_d_face_idx = self.relative_get(c_d_idx, Offset::Current).face_idx;
      let c_b_d_face_idx = self.relative_get(d_c_idx, Offset::Current).face_idx;

      self.face_refs[c_a_d_face_idx as usize] = None;
      self.face_refs[c_b_d_face_idx as usize] = None;

      self.num_removed_faces += 2;

      // reassign half_edges which had vertex c

      // TODO: consider rename
      let mut combine_twins = |first, second, edge_idx, vertex_idx| {
        // TODO: fix vertex behavior in boundary case
        if let Some(c_a_idx) =
          self.relative_get(first, Offset::Current).twin_idx
        {
          if let Some(a_d_idx) =
            self.relative_get(second, Offset::Current).twin_idx
          {
            self.vertex_refs[vertex_idx as usize]
              .as_mut()
              .unwrap()
              .half_edge_idx = a_d_idx;

            self.half_edge_refs[c_a_idx as usize].twin_idx = Some(a_d_idx);
            self.half_edge_refs[a_d_idx as usize].twin_idx = Some(c_a_idx);
            self.half_edge_refs[c_a_idx as usize].edge_idx = edge_idx;
            self.half_edge_refs[a_d_idx as usize].edge_idx = edge_idx;
          } else {
            self.half_edge_refs[c_a_idx as usize].twin_idx = None;
            self.half_edge_refs[c_a_idx as usize].edge_idx = edge_idx;
          }
        } else if let Some(a_d_idx) =
          self.relative_get(second, Offset::Current).twin_idx
        {
          self.half_edge_refs[a_d_idx as usize].twin_idx = None;
          self.half_edge_refs[a_d_idx as usize].edge_idx = edge_idx;
        }
      };

      dbg!(a_vertex_idx);
      dbg!(b_vertex_idx);
      dbg!(m_vertex_idx);

      combine_twins(a_c_idx, d_a_idx, m_a_edge_idx, a_vertex_idx);
      combine_twins(b_d_idx, c_b_idx, b_m_edge_idx, b_vertex_idx);

      self.verify_vertex_valid(m_vertex_idx);
      self.verify_edge_valid(m_a_edge_idx);
      self.verify_edge_valid(b_m_edge_idx);

      (
        [m_vertex_idx, a_vertex_idx, b_vertex_idx],
        [m_a_edge_idx, b_m_edge_idx, c_a_edge_idx, b_c_edge_idx],
      )
    } else {
      panic!("AHHHH boundary");
    }
  }

  fn set_position(&mut self, key: IndexType, position: &Vertex) {
    self.vertex_refs[key as usize].as_mut().unwrap().vertex = *position;
  }

  fn get_position(&self, key: IndexType) -> Vertex {
    self.vertex_refs[key as usize].as_ref().unwrap().vertex
  }

  fn degree(&self, vertex_idx: IndexType) -> usize {
    // TODO: optimize
    let mut neighbors = Vec::new();

    self.get_vertex_neighbors(vertex_idx, &mut neighbors);

    neighbors.len()
  }

  fn get_vertex_neighbors(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool {
    let (has_boundary, half_edge_idx) = self.get_start_iter_half_edge_idx(key);

    neighbors.clear();

    let mut half_edge_idx_in =
      self.relative_get(half_edge_idx, Offset::Next).next_idx;
    let half_edge_idx_orig_in = half_edge_idx_in;

    let mut first = true;

    neighbors.push(
      self
        .relative_get(half_edge_idx_in, Offset::Current)
        .vertex_idx,
    );

    // iterate from one side in other direction
    while first || half_edge_idx_in != half_edge_idx_orig_in {
      debug_assert_eq!(
        self.relative_get(half_edge_idx_in, Offset::Next).vertex_idx,
        key
      );

      neighbors.push(
        self
          .relative_get(half_edge_idx_in, Offset::NextNext)
          .vertex_idx,
      );
      if let Some(new_half_edge_idx_in) =
        self.relative_get(half_edge_idx_in, Offset::Next).twin_idx
      {
        half_edge_idx_in = new_half_edge_idx_in;
      } else {
        break;
      }

      first = false;
    }

    has_boundary
  }

  fn get_vertex_adjacent_faces(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool {
    let (has_boundary, half_edge_idx) = self.get_start_iter_half_edge_idx(key);

    neighbors.clear();

    let mut half_edge_idx_in =
      self.relative_get(half_edge_idx, Offset::Next).next_idx;
    let half_edge_idx_orig_in = half_edge_idx_in;

    let mut first = true;

    // iterate from one side in other direction
    while first || half_edge_idx_in != half_edge_idx_orig_in {
      neighbors.push(
        self
          .relative_get(half_edge_idx_in, Offset::Current)
          .face_idx,
      );
      if let Some(new_half_edge_idx_in) =
        self.relative_get(half_edge_idx_in, Offset::Next).twin_idx
      {
        half_edge_idx_in = new_half_edge_idx_in;
      } else {
        break;
      }

      first = false;
    }
    has_boundary
  }

  fn get_edge_neighbors(
    &self,
    key: IndexType,
  ) -> ([IndexType; 3], Option<IndexType>) {
    let half_edge = self.relative_get(
      self.edge_refs[key as usize].as_ref().unwrap().half_edge_idx,
      Offset::Current,
    );
    (
      [
        half_edge.vertex_idx,
        self
          .relative_get(half_edge.next_idx, Offset::Current)
          .vertex_idx,
        self
          .relative_get(half_edge.next_idx, Offset::Next)
          .vertex_idx,
      ],
      half_edge.twin_idx.map(|twin_idx| {
        self.relative_get(twin_idx, Offset::NextNext).vertex_idx
      }),
    )
  }

  fn get_endpoints(&self, key: IndexType) -> [IndexType; 2] {
    let half_edge = self.relative_get(
      self.edge_refs[key as usize].as_ref().unwrap().half_edge_idx,
      Offset::Current,
    );
    [
      half_edge.vertex_idx,
      self
        .relative_get(half_edge.next_idx, Offset::Current)
        .vertex_idx,
    ]
  }

  fn get_face_neighbors(&self, key: IndexType) -> [IndexType; 3] {
    self.get_face_ref_neighbors(self.face_refs[key as usize].as_ref().unwrap())
  }

  fn to_vecs(self) -> (Vec<Vertex>, Vec<Face>) {
    // TODO: GROSS
    let mut exclusive_sum = Vec::with_capacity(self.vertex_refs.len());

    let mut sum = 0;

    for vertex in &self.vertex_refs {
      exclusive_sum.push(sum);
      if vertex.is_some() {
        sum += 1;
      }
    }

    (
      self
        .vertex_refs
        .iter()
        .filter_map(|v| v.as_ref())
        .map(|v| v.vertex)
        .collect(),
      (0..(self.face_refs.len() as IndexType))
        .map(|i| &self.face_refs[i as usize])
        .filter_map(|x| x.as_ref())
        .map(|v| {
          self
            .get_face_ref_neighbors(&v)
            .iter()
            .map(|index| exclusive_sum[*index as usize])
            .collect::<Vec<IndexType>>()[..]
            .try_into()
            .unwrap()
        })
        .collect(),
    )
  }
}
