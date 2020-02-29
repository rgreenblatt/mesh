use clap::Clap;
use std::path::Path;
use tobj;

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
  Subdivide(SubdivideOpts),
  #[clap(name = "simplify")]
  Simplify(SimplifyOpts),
  #[clap(name = "remesh")]
  Remesh(RemeshOpts),
  #[clap(name = "denoise")]
  Denoise(DenoiseOpts),
}

#[derive(Clap)]
struct SubdivideOpts {
  iterations: u32,
}

#[derive(Clap)]
struct SimplifyOpts {
  faces_to_remove: u32,
}

#[derive(Clap)]
struct RemeshOpts {
  smoothing_weight: f32,
}

#[derive(Clap)]
struct DenoiseOpts {
  sigma_c: f32,
  sigma_s: f32,
  kernel_size: u32,
}

fn main() {
  let opts: Opts = Opts::parse();

  let mesh = tobj::load_obj(&Path::new(&opts.infile));
}
