use nalgebra::base::Vector3;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use tobj;

pub struct Mesh {
  pub vertices: Vec<Vector3<f32>>,
  pub faces: Vec<[u32; 3]>,
}

impl Mesh {
  pub fn new(path: &Path) -> Result<Self, tobj::LoadError> {
    let (models, _) = tobj::load_obj(path)?;
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for m in models {
      let mesh = &m.mesh;
      vertices.extend(
        mesh
          .positions
          .chunks_exact(3)
          .map(|vals| Vector3::new(vals[0], vals[1], vals[2])),
      );
      faces.extend(
        mesh
          .indices
          .chunks_exact(3)
          .map(|vals| [vals[0], vals[1], vals[2]]),
      );
    }

    println!(
      "loaded {} faces and {} vertices",
      faces.len(),
      vertices.len()
    );

    Ok(Mesh { vertices, faces })
  }

  pub fn save(self: &Mesh, path: &Path) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);

    for vertex in &self.vertices {
      write!(&mut writer, "v {} {} {}\n", vertex[0], vertex[1], vertex[2])?;
    }

    for face in &self.faces {
      write!(
        &mut writer,
        "f {} {} {}\n",
        face[0] + 1,
        face[1] + 1,
        face[2] + 1
      )?;
    }

    Ok(())
  }
}
