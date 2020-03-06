use crate::data_structure::base::Face;
use crate::data_structure::base::IndexType;
use crate::data_structure::base::Vector3;
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
  vertex: Vector3,
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
  removed_half_edges: HashSet<u32>, // only used for debugging
}

enum Offset {
  Current,
  Next,
  NextNext,
}

struct IterVertexHalfEdges<'a> {
  vertex_idx: IndexType,
  half_edge_idx: Option<IndexType>,
  half_edge_idx_orig: IndexType,
  first: bool,
  half_edge: &'a HalfEdge,
}

impl<'a> Iterator for IterVertexHalfEdges<'a> {
  type Item = IndexType;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(half_edge_idx) = self.half_edge_idx {
      if self.first || half_edge_idx != self.half_edge_idx_orig {
        let out = Some(half_edge_idx);
        debug_assert!(!self
          .half_edge
          .removed_half_edges
          .contains(&half_edge_idx));
        debug_assert!(!self.half_edge.removed_half_edges.contains(
          &self
            .half_edge
            .relative_get(half_edge_idx, Offset::Current)
            .next_idx
        ));
        debug_assert!(!self.half_edge.removed_half_edges.contains(
          &self
            .half_edge
            .relative_get(half_edge_idx, Offset::Next)
            .next_idx
        ));
        debug_assert_eq!(
          self
            .half_edge
            .relative_get(half_edge_idx, Offset::Current)
            .vertex_idx,
          self.vertex_idx
        );

        self.half_edge_idx = self
          .half_edge
          .relative_get(half_edge_idx, Offset::NextNext)
          .twin_idx;

        self.first = false;

        out
      } else {
        None
      }
    } else {
      None
    }
  }
}

impl HalfEdge {
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
      debug_assert_eq!(
        self.relative_get(half_edge_idx, Offset::Current).vertex_idx,
        vertex_idx
      );

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

  fn vertex_half_edges<'a>(
    &'a self,
    vertex_idx: IndexType,
  ) -> (bool, impl Iterator<Item = IndexType> + 'a) {
    let (has_boundary, half_edge_idx) =
      self.get_start_iter_half_edge_idx(vertex_idx);

    debug_assert_eq!(
      self.relative_get(half_edge_idx, Offset::Current).vertex_idx,
      vertex_idx
    );

    let half_edge_idx_orig = half_edge_idx;

    (
      has_boundary,
      IterVertexHalfEdges {
        vertex_idx,
        half_edge_idx: Some(half_edge_idx),
        half_edge_idx_orig,
        first: true,
        half_edge: self,
      },
    )
  }

  fn combine_twins(
    &mut self,
    first_half_edge_idx: IndexType,
    second_half_edge_idx: IndexType,
    edge_idx: IndexType,
    vertex_idx: IndexType,
  ) {
    // TODO: fix vertex and edge behavior in boundary case
    if let Some(c_a_idx) = self
      .relative_get(first_half_edge_idx, Offset::Current)
      .twin_idx
    {
      if let Some(a_d_idx) = self
        .relative_get(second_half_edge_idx, Offset::Current)
        .twin_idx
      {
        self.vertex_refs[vertex_idx as usize]
          .as_mut()
          .unwrap()
          .half_edge_idx = a_d_idx;

        self.edge_refs[edge_idx as usize]
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
    } else if let Some(a_d_idx) = self
      .relative_get(second_half_edge_idx, Offset::Current)
      .twin_idx
    {
      self.half_edge_refs[a_d_idx as usize].twin_idx = None;
      self.half_edge_refs[a_d_idx as usize].edge_idx = edge_idx;
    }
  }

  fn verify_half_edge_valid(&self, half_edge_idx: IndexType) {
    if cfg!(debug_assertions) {
      let half_edge = &self.half_edge_refs[half_edge_idx as usize];
      debug_assert!(self.vertex_refs[half_edge.vertex_idx as usize].is_some());
      debug_assert!(self.edge_refs[half_edge.edge_idx as usize].is_some());
      debug_assert!(self.face_refs[half_edge.face_idx as usize].is_some());
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

  fn verify_vertex_valid(&self, vertex_idx: IndexType) {
    if cfg!(debug_assertions) {
      debug_assert_eq!(
        vertex_idx,
        self.half_edge_refs[self.vertex_refs[vertex_idx as usize]
          .as_ref()
          .unwrap()
          .half_edge_idx as usize]
          .vertex_idx
      );

      let mut neighbors = Vec::new();
      self.get_vertex_neighbors(vertex_idx, &mut neighbors);
      debug_assert!(!neighbors.contains(&vertex_idx));
      self.verify_half_edge_valid(
        self.vertex_refs[vertex_idx as usize]
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

  fn verify_edge_valid(
    &self,
    edge_idx: IndexType,
    first_vertex_idx: IndexType,
    second_vertex_idx: IndexType,
  ) {
    if cfg!(debug_assertions) {
      let [left, right] = self.get_endpoints(edge_idx);

      let vertex_hash_set =
        HashSet::<IndexType>::from_iter([left, right].iter().cloned());

      debug_assert_eq!(vertex_hash_set.len(), 2);
      debug_assert_eq!(
        vertex_hash_set,
        HashSet::<IndexType>::from_iter(
          [first_vertex_idx, second_vertex_idx].iter().cloned()
        )
      );

      debug_assert_eq!(
        edge_idx,
        self.half_edge_refs[self.edge_refs[edge_idx as usize]
          .as_ref()
          .unwrap()
          .half_edge_idx as usize]
          .edge_idx
      );

      self.verify_vertex_valid(left);
      self.verify_vertex_valid(right);
    }
  }

  fn verify_face_valid(
    &self,
    face_idx: IndexType,
    first_vertex_idx: IndexType,
    second_vertex_idx: IndexType,
    third_vertex_idx: IndexType,
  ) {
    if cfg!(debug_assertions) {
      let [first, second, third] = self.get_face_neighbors(face_idx);

      let vertex_hash_set =
        HashSet::<IndexType>::from_iter([first, second, third].iter().cloned());

      debug_assert_eq!(vertex_hash_set.len(), 3);
      debug_assert_eq!(
        vertex_hash_set,
        HashSet::<IndexType>::from_iter(
          vec![first_vertex_idx, second_vertex_idx, third_vertex_idx]
            .iter()
            .cloned()
        )
      );

      debug_assert_eq!(
        face_idx,
        self.half_edge_refs[self.face_refs[face_idx as usize]
          .as_ref()
          .unwrap()
          .half_edge_idx as usize]
          .face_idx
      );

      self.verify_vertex_valid(first);
      self.verify_vertex_valid(second);
      self.verify_vertex_valid(third);
    }
  }

  fn check_all(&self) {
    if cfg!(debug_assertions) {
      let vertex_iter = self
        .vertex_refs
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)));

      let mut true_count = 0;
      for (vertex_idx, _) in vertex_iter {
        true_count += 1;
        self.verify_vertex_valid(vertex_idx.try_into().unwrap());
      }

      assert_eq!(true_count, self.num_vertices());

      let mut edge_hash_set = HashSet::new();

      let edge_iter = self
        .edge_refs
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)));

      let mut true_count = 0;
      for (edge_idx, _) in edge_iter {
        true_count += 1;
        let edge_idx = edge_idx.try_into().unwrap();
        let [l, r] = self.get_endpoints(edge_idx);

        debug_assert!(!edge_hash_set.contains(&(l, r)));
        debug_assert!(!edge_hash_set.contains(&(r, l)));
        edge_hash_set.insert((l, r));
        debug_assert!(edge_hash_set.contains(&(l, r)));

        self.verify_edge_valid(edge_idx, l, r);
      }

      assert_eq!(true_count, self.num_edges());

      let face_iter = self
        .face_refs
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)));

      let mut true_count = 0;
      for (face_idx, _) in face_iter {
        true_count += 1;
        let face_idx = face_idx.try_into().unwrap();
        let [v_0, v_1, v_2] = self.get_face_neighbors(face_idx);

        let check_edge_exists = |v_l, v_r| {
          debug_assert!(
            edge_hash_set.contains(&(v_l, v_r))
              ^ edge_hash_set.contains(&(v_r, v_l))
          );
        };

        check_edge_exists(v_0, v_1);
        check_edge_exists(v_0, v_2);
        check_edge_exists(v_1, v_2);

        self.verify_face_valid(face_idx, v_0, v_1, v_2);
      }

      assert_eq!(true_count, self.num_faces());

      let half_edge_iter =
        self.half_edge_refs.iter().enumerate().filter_map(|(i, x)| {
          if self.removed_half_edges.contains(&(i.try_into().unwrap())) {
            None
          } else {
            Some((i, x))
          }
        });

      for (half_edge_idx, _) in half_edge_iter {
        self.verify_half_edge_valid(half_edge_idx.try_into().unwrap());
      }
    }
  }
}

impl DataStructure for HalfEdge {
  fn from_iters<IterVert, IterFace>(vertices: IterVert, faces: IterFace) -> Self
  where
    IterVert: IntoIterator<Item = Vector3>,
    IterFace: IntoIterator<Item = Face>,
  {
    let vertices_vec: Vec<Vector3> = vertices.into_iter().collect();

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

    let out = HalfEdge {
      half_edge_refs,
      vertex_refs,
      edge_refs,
      face_refs,
      num_removed_vertices: 0,
      num_removed_edges: 0,
      num_removed_faces: 0,
      removed_half_edges: HashSet::new(),
    };

    out.check_all();

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

  fn flip_edge(&mut self, key: IndexType) -> Option<()> {
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

      Some(())
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

  // Very complex I'm afraid...
  #[allow(clippy::cognitive_complexity)]
  fn collapse_edge(
    &mut self,
    key: IndexType,
    modified_edges: &mut Vec<(IndexType, IndexType)>,
    removed_edges: &mut Vec<IndexType>,
  ) -> Option<IndexType> {
    modified_edges.clear();
    removed_edges.clear();

    // see page 28 of lecture slides "meshes_geoprocessing" for
    // a,b,c,d,m reference

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

      // SPEED: too much memory allocation inside hot portion...
      let mut store = Vec::new();
      self.get_vertex_neighbors(c_vertex_idx, &mut store);
      let c_neighbors = HashSet::<IndexType>::from_iter(store);
      let mut store = Vec::new();
      self.get_vertex_neighbors(d_vertex_idx, &mut store);
      let d_neighbors = HashSet::<IndexType>::from_iter(store);

      let mut num_common = 0;

      for vertex_idx in c_neighbors.intersection(&d_neighbors) {
        num_common += 1;
        if self.degree(*vertex_idx) <= 3 {
          return None;
        }
      }

      if num_common > 2 {
        return None;
      }

      let m_vertex_idx = d_vertex_idx;

      let (_, half_edge_iter) = self.vertex_half_edges(d_vertex_idx);

      for half_edge_idx in half_edge_iter {
        debug_assert_eq!(
          self.relative_get(half_edge_idx, Offset::Current).vertex_idx,
          d_vertex_idx
        );

        let other_edge_vertex =
          self.relative_get(half_edge_idx, Offset::Next).vertex_idx;

        let edge = self.relative_get(half_edge_idx, Offset::Current).edge_idx;

        if other_edge_vertex != c_vertex_idx {
          modified_edges.push((edge, other_edge_vertex));
        }
      }

      // this can't use iter because it must mutate
      {
        let (_, half_edge_idx) =
          self.get_start_iter_half_edge_idx(c_vertex_idx);

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

          let other_edge_vertex =
            self.relative_get(this_edge_idx, Offset::Next).vertex_idx;

          debug_assert!(c_neighbors.contains(&other_edge_vertex));

          let edge = self.relative_get(this_edge_idx, Offset::Current).edge_idx;

          if d_neighbors.contains(&other_edge_vertex)
            || other_edge_vertex == d_vertex_idx
          {
            removed_edges.push(edge);
            self.edge_refs[edge as usize] = None;
            if cfg!(debug_assertions)
              && other_edge_vertex != a_vertex_idx
              && other_edge_vertex != b_vertex_idx
            {
              self.removed_half_edges.insert(this_edge_idx);

              if let Some(twin_idx) =
                self.relative_get(this_edge_idx, Offset::Current).twin_idx
              {
                self.removed_half_edges.insert(twin_idx);
              }
            }
          } else {
            modified_edges.push((edge, other_edge_vertex));
          }

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

      if cfg!(debug_assertions) {
        self.removed_half_edges.insert(d_a_idx);
        self.removed_half_edges.insert(b_d_idx);
        self.removed_half_edges.insert(c_d_idx);
        self.removed_half_edges.insert(a_c_idx);
        self.removed_half_edges.insert(c_b_idx);
        self.removed_half_edges.insert(d_c_idx);
      }

      self.vertex_refs[c_vertex_idx as usize] = None;

      self.num_removed_vertices += 1;

      // TODO (boundary)
      let m_b_idx = self.relative_get(b_d_idx, Offset::Current).twin_idx;

      assert!(m_b_idx.is_some());

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

      if cfg!(debug_assertions) {
        // c<->a removed
        let c_a_edge_idx = self.relative_get(a_c_idx, Offset::Current).edge_idx;
        // b<->c removed
        let b_c_edge_idx = self.relative_get(c_b_idx, Offset::Current).edge_idx;
        // c<->d removed
        let c_d_edge_idx = self.relative_get(c_d_idx, Offset::Current).edge_idx;

        debug_assert!(self.edge_refs[c_a_edge_idx as usize].is_none());
        debug_assert!(self.edge_refs[b_c_edge_idx as usize].is_none());
        debug_assert!(self.edge_refs[c_d_edge_idx as usize].is_none());

        debug_assert!(self.edge_refs[d_a_edge_idx as usize].is_some());
        debug_assert!(self.edge_refs[b_d_edge_idx as usize].is_some());
      }

      self.num_removed_edges += 3;

      // faces are removed
      let c_a_d_face_idx = self.relative_get(c_d_idx, Offset::Current).face_idx;
      let c_b_d_face_idx = self.relative_get(d_c_idx, Offset::Current).face_idx;

      self.face_refs[c_a_d_face_idx as usize] = None;
      self.face_refs[c_b_d_face_idx as usize] = None;

      self.num_removed_faces += 2;

      // reassign half_edges which had vertex c
      self.combine_twins(a_c_idx, d_a_idx, m_a_edge_idx, a_vertex_idx);
      self.combine_twins(b_d_idx, c_b_idx, b_m_edge_idx, b_vertex_idx);

      self.verify_vertex_valid(m_vertex_idx);
      self.verify_vertex_valid(a_vertex_idx);
      self.verify_vertex_valid(b_vertex_idx);
      self.verify_edge_valid(m_a_edge_idx, m_vertex_idx, a_vertex_idx);
      self.verify_edge_valid(b_m_edge_idx, m_vertex_idx, b_vertex_idx);

      debug_assert!(modified_edges.contains(&(m_a_edge_idx, a_vertex_idx)));
      debug_assert!(modified_edges.contains(&(b_m_edge_idx, b_vertex_idx)));

      Some(m_vertex_idx)
    } else {
      panic!("AHHHH boundary");
    }
  }

  fn set_position(&mut self, key: IndexType, position: &Vector3) {
    self.vertex_refs[key as usize].as_mut().unwrap().vertex = *position;
  }

  fn get_position(&self, key: IndexType) -> Vector3 {
    self.vertex_refs[key as usize].as_ref().unwrap().vertex
  }

  fn degree(&self, vertex_idx: IndexType) -> usize {
    self.vertex_half_edges(vertex_idx).1.count()
  }

  fn get_vertex_neighbors_append(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool {
    let (has_boundary, half_edge_iter) = self.vertex_half_edges(key);

    for half_edge_idx in half_edge_iter {
      neighbors.push(self.relative_get(half_edge_idx, Offset::Next).vertex_idx);
    }

    has_boundary
  }

  fn get_vertex_adjacent_faces(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool {
    neighbors.clear();

    let (has_boundary, half_edge_iter) = self.vertex_half_edges(key);

    for half_edge_idx in half_edge_iter {
      neighbors
        .push(self.relative_get(half_edge_idx, Offset::Current).face_idx);
    }

    debug_assert_eq!(
      HashSet::<IndexType>::from_iter(neighbors.iter().cloned()).len(),
      neighbors.len()
    );

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

  fn to_vecs(self) -> (Vec<Vector3>, Vec<Face>) {
    self.check_all();

    // SPEED: GROSS
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
