pub mod data_structure;
pub mod mesh_operation;
pub mod utils;

pub use utils::get_normal;

pub use data_structure::DataStructure;
pub use data_structure::HalfEdge;
pub use data_structure::IndexType;
pub use data_structure::Vector3;

pub use mesh_operation::Operation;

pub use mesh_operation::Denoise;
pub use mesh_operation::Noise;
pub use mesh_operation::Remesh;
pub use mesh_operation::Simplify;
pub use mesh_operation::Subdivide;
