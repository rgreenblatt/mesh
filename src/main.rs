use clap::Clap;
use std::path::Path;

use mesh::Denoise;
use mesh::HalfEdge;
use mesh::DataStructure;
use mesh::Operation;
use mesh::Remesh;
use mesh::Simplify;
use mesh::Subdivide;

#[derive(Clap)]
#[clap(version = "0.1", author = "Ryan G.")]
struct Opts {
  /// Input mesh file
  infile: String,
  /// Output mesh file
  outfile: String,

  #[clap(subcommand)]
  /// method
  method: Methods,
}

#[derive(Clap)]
enum Methods {
  #[clap(name = "subdivide")]
  /// subdivide the mesh using loop subdivision
  Subdivide(Subdivide),
  #[clap(name = "simplify")]
  Simplify(Simplify),
  #[clap(name = "remesh")]
  Remesh(Remesh),
  #[clap(name = "denoise")]
  Denoise(Denoise),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let opts: Opts = Opts::parse();
  let mut mesh = HalfEdge::from_obj(&Path::new(&opts.infile))?;

  match opts.method {
    Methods::Subdivide(v) => v.apply(&mut mesh),
    Methods::Simplify(v) => v.apply(&mut mesh),
    Methods::Remesh(v) => v.apply(&mut mesh),
    Methods::Denoise(v) => v.apply(&mut mesh),
  };

  mesh.save_obj(&Path::new(&opts.outfile))?;

  Ok(())
}
