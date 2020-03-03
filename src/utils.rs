use crate::Vector3;

pub fn get_normal(vertices: [Vector3; 3]) -> Vector3 {
  ((vertices[1] - vertices[0]).cross(&(vertices[2] - vertices[0]))).normalize()
}
