use std::fs::File;
use std::hash::Hash;
use std::io::{prelude::*, BufWriter};
use std::path::Path;

use nalgebra::base::Vector3;

pub type Vertex = Vector3<f32>;
pub type Face = [u32; 3];

pub trait DataStructure: Sized {
  fn from_iters<IterVert, IterFace>(
    vertices: IterVert,
    faces: IterFace,
  ) -> Self
  where
    IterVert: IntoIterator<Item = Vertex>,
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

  type VertexIdx: Sized + std::fmt::Debug + Eq + Copy + Hash;

  type EdgeIdx: Sized + std::fmt::Debug + Eq + Copy + Hash;

  type FaceIdx: Sized + std::fmt::Debug + Eq + Copy + Hash;

  fn initial_vertex(&self) -> Option<Self::VertexIdx>;

  fn next_vertex(&self, key: Self::VertexIdx) -> Option<Self::VertexIdx>;

  fn initial_edge(&self) -> Option<Self::EdgeIdx>;

  fn next_edge(&self, key: Self::EdgeIdx) -> Option<Self::EdgeIdx>;

  // fn initial_face(&self) -> Option<Self::FaceIdx>;

  // fn next_face(&self, key: Self::FaceIdx) -> Option<Self::FaceIdx>;

  fn flip_edge(&mut self, key: Self::EdgeIdx);

  // order of returned edges:
  // original edge left, original edge right
  // new edge top, new edge bottom (same order as get_opposite_points)
  fn split_edge(
    &mut self,
    key: Self::EdgeIdx,
  ) -> (Self::VertexIdx, [Self::EdgeIdx; 4]);

  // new vertex
  // modified edge, modified edge
  // removed edge, removed edge
  fn collapse_edge(
    &mut self,
    key: Self::EdgeIdx,
  ) -> (Self::VertexIdx, [Self::EdgeIdx; 4]);

  fn set_position(&mut self, key: Self::VertexIdx, position: &Vertex);

  fn get_position(&self, key: Self::VertexIdx) -> Vertex;

  // first is next to second is next to third...
  // return value is if there is discontinutity...
  fn get_vertex_neighbors(
    &self,
    key: Self::VertexIdx,
    neighbors: &mut Vec<Self::VertexIdx>,
  ) -> bool;

  // endpoint, endpoint, and far points of adjacent faces
  fn get_edge_neighbors(
    &self,
    key: Self::EdgeIdx,
  ) -> ([Self::VertexIdx; 3], Option<Self::VertexIdx>);

  // fn get_face_neighbors(
  //   &self,
  //   key: Self::FaceIdx,
  //   neighbors: &mut Vec<Self::VertexIdx>,
  // );

  fn get_endpoints(&self, key: Self::EdgeIdx) -> [Self::VertexIdx; 2];

  // fn get_face_edges(&self, key: Self::FaceIdx) -> [Self::FaceIdx; 3];

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

  fn to_vecs(self) -> (Vec<Vertex>, Vec<Face>);
}
