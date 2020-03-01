use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
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

  fn num_vertices(&self) -> usize;

  fn num_edges(&self) -> usize;

  fn num_faces(&self) -> usize;

  type VertexKey: Sized + std::fmt::Debug + Eq;
  type EdgeKey: Sized + std::fmt::Debug + Eq;
  // type FaceKey: Sized + std::fmt::Debug + Eq;
  type IterVertexKeys: Iterator<Item = Self::VertexKey>;
  type IterEdgeKeys: Iterator<Item = Self::EdgeKey>;
  // type IterFaceKeys: Iterator<Item = Self::FaceKey>;

  fn vertex_keys(&self) -> Self::IterVertexKeys;

  fn edge_keys(&self) -> Self::IterEdgeKeys;

  // fn face_keys(&self) -> Self::IterFaceKeys;

  fn flip_edge(&mut self, key: &Self::EdgeKey);

  // order of returned edges:
  // original edge left, original edge right
  // new edge top, new edge bottom (same order as get_opposite_points)
  fn split_edge(
    &mut self,
    key: &Self::EdgeKey,
  ) -> (Self::VertexKey, [Self::EdgeKey; 4]);

  // fn collapse_edge(&mut self, key: &Self::EdgeKey) -> Self::VertexKey;

  fn set_position(&mut self, key: &Self::VertexKey, position: &Vertex);

  fn get_position(&self, key: &Self::VertexKey) -> Vertex;

  fn get_vertex_neighbors(
    &self,
    key: &Self::VertexKey,
    neighbors: &mut Vec<Self::VertexKey>,
  );

  // endpoint, endpoint, and far points of adjacent faces
  fn get_edge_neighbors(
    &self,
    key: &Self::EdgeKey,
  ) -> ([Self::VertexKey; 3], Option<Self::VertexKey>);

  // fn get_face_neighbors(
  //   &self,
  //   key: &Self::FaceKey,
  //   neighbors: &mut Vec<Self::VertexKey>,
  // );

  fn get_endpoints(&self, key: &Self::EdgeKey) -> [Self::VertexKey; 2];

  // fn get_face_edges(&self, key: &Self::FaceKey) -> [Self::FaceKey; 3];

  fn to_vecs(&mut self) -> (Vec<Vertex>, Vec<Face>);

  fn save_obj(&mut self, path: &Path) -> std::io::Result<()> {
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
      )?
    }

    writer.flush()?;

    Ok(())
  }
}
