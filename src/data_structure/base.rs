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

  type EdgeKey : Sized;
  type VertexKey : Sized;

  // TODO: gross... why can't iterators be used...
  fn initial_edge(&self) -> Self::EdgeKey;

  fn next_edge(&self, key : &Self::EdgeKey) -> Self::EdgeKey;

  fn flip_edge(&mut self, key: &Self::EdgeKey);

  // fn split_edge(&mut self, key: &Self::EdgeKey) -> Self::VertexKey;

  // fn collapse_edge(&mut self, key: &Self::EdgeKey) -> Self::VertexKey;

  // fn set_position(&mut self, key: &Self::VertexKey, position: &Vertex);

  // fn get_position(&self, key: &Self::VertexKey) -> Vertex;

  // fn get_neighbors(
  //   &self,
  //   key: &Self::VertexKey,
  //   neighbors: &mut Vec<Self::VertexKey>,
  // );

  // fn get_endpoints(&self, key : &Self::EdgeKey) -> [Self::VertexKey; 2];

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