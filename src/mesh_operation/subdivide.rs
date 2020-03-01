use crate::mesh_operation::Operation;
use crate::DataStructure;
use crate::Vertex;
use clap::Clap;

#[derive(Clap)]
pub struct Subdivide {
  iterations: u32,
}

impl Operation for Subdivide {
  fn apply<D: DataStructure>(&self, mesh: &mut D) {
    for _ in 0..self.iterations {
      let mut new_vertex_info = Vec::with_capacity(mesh.num_edges());

      for edge_key in mesh.edge_keys() {
        let ([near_0, near_1, far_0], far_op) =
          mesh.get_edge_neighbors(&edge_key);

        let far_1 = far_op.expect("TODO: handle boundaries");

        let weight_near = 3.0 / 8.0;
        let weight_far = 1.0 / 8.0;
        let pos = weight_near
          * (mesh.get_position(&near_0) + mesh.get_position(&near_1))
          + weight_far
            * (mesh.get_position(&far_0) + mesh.get_position(&far_1));

        new_vertex_info.push((edge_key, pos, [far_0, far_1]));
      }

      let mut neighbors = Vec::new();

      // set positions of old vertices
      for vertex_key in mesh.vertex_keys() {
        mesh.get_vertex_neighbors(&vertex_key, &mut neighbors);

        let n = neighbors.len() as f32;

        let u = if neighbors.len() == 3 {
          3.0 / 16.0
        } else {
          3.0 / (8.0 * n)
        };

        let avg =
          neighbors.iter().fold(Vertex::new(0.0, 0.0, 0.0), |acc, x| {
            debug_assert!(*x != vertex_key);

            acc + mesh.get_position(x)
          }) * u;

        let pos = avg
          + mesh
            .get_position(&vertex_key)
            .component_mul(&Vertex::from_element(1.0 - n * u));

        mesh.set_position(&vertex_key, &pos);
      }

      let mut to_flip = Vec::with_capacity(mesh.num_faces());

      let faces_before = mesh.num_faces();


      // collect so we don't iterate over new edges... (there are
      // more efficient approaches...)
      for (edge_key, vertex_pos, [far_l, far_r]) in new_vertex_info {
        let (new_vertex, [new_l, new_r, _, _]) = mesh.split_edge(&edge_key);

        mesh.set_position(&new_vertex, &vertex_pos);

        let [l_p_0, l_p_1] = mesh.get_endpoints(&new_l);
        let [r_p_0, r_p_1] = mesh.get_endpoints(&new_r);

        debug_assert!(l_p_0 == new_vertex || l_p_1 == new_vertex);
        debug_assert!(r_p_0 == new_vertex || r_p_1 == new_vertex);
        debug_assert!(l_p_0 != far_r && l_p_1 != far_r);
        debug_assert!(r_p_0 != far_l && r_p_1 != far_l);

        if l_p_0 == far_l || l_p_1 == far_l {
          to_flip.push(new_l);
        }

        if r_p_0 == far_r || r_p_1 == far_r {
          to_flip.push(new_r);
        }
      }

      assert_eq!(to_flip.len(), faces_before);

      for flip_edge in to_flip {
        mesh.flip_edge(&flip_edge);
      }
    }
  }
}
