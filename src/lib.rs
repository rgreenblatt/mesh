#![feature(type_alias_impl_trait)]

pub mod data_structure;
pub mod mesh_operation;

pub use data_structure::DataStructure;
pub use data_structure::HalfEdge;
pub use data_structure::Vertex;

pub use mesh_operation::Operation;

pub use mesh_operation::Denoise;
pub use mesh_operation::Remesh;
pub use mesh_operation::Simplify;
pub use mesh_operation::Subdivide;
