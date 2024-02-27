use crate::douconel::Douconel;
use crate::douconel_embedded::{HasNormal, HasPosition};
use glam::Vec3;
use itertools::Itertools;
use simple_error::bail;
use std::error::Error;
use std::fs::OpenOptions;

// Read an STL file from `path`, and construct a DCEL.
impl<V: Default + HasPosition, E: Default, F: Default + HasNormal> Douconel<V, E, F> {
    pub fn from_stl(path: &str) -> Result<Self, Box<dyn Error>> {
        let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

        let faces = stl.faces.iter().map(|f| f.vertices.to_vec()).collect_vec();

        let res = Self::from_faces(faces);

        if let Ok((mut douconel, vertex_map, face_map)) = res {
            for (inp_vertex_id, inp_vertex_pos) in stl.vertices.iter().enumerate() {
                let vert_id = vertex_map.get_by_left(&inp_vertex_id).copied().unwrap();
                if let Some(v) = douconel.verts.get_mut(vert_id) {
                    v.set_position(Vec3::new(
                        inp_vertex_pos[0],
                        inp_vertex_pos[1],
                        inp_vertex_pos[2],
                    ));
                }
            }
            for (inp_face_id, inp_face) in stl.faces.iter().enumerate() {
                let face_id = face_map.get_by_left(&inp_face_id).copied().unwrap();
                if let Some(f) = douconel.faces.get_mut(face_id) {
                    f.set_normal(Vec3::new(
                        inp_face.normal[0],
                        inp_face.normal[1],
                        inp_face.normal[2],
                    ));
                }
            }

            Ok(douconel)
        } else {
            bail!(res.err().unwrap())
        }
    }
}
