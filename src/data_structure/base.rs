use std::fs::File;
use std::io::{prelude::*, BufWriter};
use std::path::Path;

use crate::get_normal;

use nalgebra;

pub type Vector3 = nalgebra::base::Vector3<f32>;
pub type Face = [u32; 3];
pub type IndexType = u32;

pub trait DataStructure: Sized {
  fn from_iters<IterVert, IterFace>(
    vertices: IterVert,
    faces: IterFace,
  ) -> Self
  where
    IterVert: IntoIterator<Item = Vector3>,
    IterFace: IntoIterator<Item = Face>;

  fn from_obj(path: &Path) -> Result<Self, tobj::LoadError> {
    let (models, _) = tobj::load_obj(path)?;
    let vertices = models
      .iter()
      .map(|m| {
        let mesh = &m.mesh;

        mesh
          .positions
          .chunks_exact(3)
          .map(|vals| Vector3::new(vals[0], vals[1], vals[2]))
      })
      .flatten();

    let faces = models
      .iter()
      .map(|m| {
        let mesh = &m.mesh;

        mesh
          .indices
          .chunks_exact(3)
          .map(|vals| [vals[0], vals[1], vals[2]])
      })
      .flatten();

    Ok(Self::from_iters(vertices, faces))
  }

  fn max_idx_vertices(&self) -> usize;

  fn max_idx_edges(&self) -> usize;

  fn max_idx_faces(&self) -> usize;

  fn num_vertices(&self) -> usize;

  fn num_edges(&self) -> usize;

  fn num_faces(&self) -> usize;

  // TODO: generic?
  fn initial_vertex(&self) -> Option<IndexType>;

  fn next_vertex(&self, key: IndexType) -> Option<IndexType>;

  fn initial_edge(&self) -> Option<IndexType>;

  fn next_edge(&self, key: IndexType) -> Option<IndexType>;

  fn initial_face(&self) -> Option<IndexType>;

  fn next_face(&self, key: IndexType) -> Option<IndexType>;

  fn flip_edge(&mut self, key: IndexType) -> Option<()>;

  // order of returned edges:
  // original edge left, original edge right
  // new edge top, new edge bottom (same order as get_opposite_points)
  fn split_edge(&mut self, key: IndexType) -> (IndexType, [IndexType; 4]);

  // TODO:
  // new vertex, new vertex for modified 0, new vertex for modified 1
  fn collapse_edge(
    &mut self,
    key: IndexType,
    // edge idx and other vertex
    modified_edges : &mut Vec<(IndexType, IndexType)>,
    removed_edges : &mut Vec<IndexType>,
  ) -> Option<IndexType>;

  fn set_position(&mut self, key: IndexType, position: &Vector3);

  fn get_position(&self, key: IndexType) -> Vector3;

  fn degree(&self, vertex_idx: IndexType) -> usize;

  // first is next to second is next to third...
  // return value is if there is discontinutity...
  fn get_vertex_neighbors(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool {
    neighbors.clear();

    self.get_vertex_neighbors_append(key, neighbors)
  }

  // same as get vertex neighbors, but appends to vec instead of clearing
  fn get_vertex_neighbors_append(
    &self,
    key: IndexType,
    neighbors: &mut Vec<IndexType>,
  ) -> bool;

  fn get_vertex_adjacent_faces(
    &self,
    key: IndexType,
    faces: &mut Vec<IndexType>,
  ) -> bool;

  // endpoint, endpoint, and far points of adjacent faces
  fn get_edge_neighbors(
    &self,
    key: IndexType,
  ) -> ([IndexType; 3], Option<IndexType>);

  fn get_face_neighbors(&self, key: IndexType) -> [IndexType; 3];

  // normal and positions of each vertex
  fn get_face_normal(&self, face_idx: IndexType) -> (Vector3, [Vector3; 3]) {
    let [v_0, v_1, v_2] = self.get_face_neighbors(face_idx);

    let p_0 = self.get_position(v_0);
    let p_1 = self.get_position(v_1);
    let p_2 = self.get_position(v_2);

    let arr = [p_0, p_1, p_2];

    let normal = get_normal(arr);

    (normal, arr)
  }

  // normal and positions of the each vertex
  fn get_vertex_normal(
    &self,
    vertex_idx: IndexType,
    vertex_adjacent_faces: &mut Vec<IndexType>,
  ) -> Vector3 {
    self.get_vertex_adjacent_faces(vertex_idx, vertex_adjacent_faces);

    vertex_adjacent_faces
      .iter()
      .fold(Vector3::zeros(), |acc, face_idx| {
        acc + self.get_face_normal(*face_idx).0
      }).normalize()
  }

  fn get_endpoints(&self, key: IndexType) -> [IndexType; 2];

  fn save_obj(self, path: &Path) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);

    let (vertices, faces) = self.to_vecs();

    for vertex in vertices {
      writeln!(&mut writer, "v {} {} {}", vertex[0], vertex[1], vertex[2])?
    }

    for face in faces {
      writeln!(
        &mut writer,
        "f {} {} {}",
        face[0] + 1,
        face[1] + 1,
        face[2] + 1
      )?;
    }

    writer.flush()?;

    Ok(())
  }

  fn to_vecs(self) -> (Vec<Vector3>, Vec<Face>);
}
