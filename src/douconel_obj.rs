use crate::douconel::{Douconel, FaceID, VertID};
use crate::douconel_embedded::{HasNormal, HasPosition};
use bimap::BiHashMap;
use glam::Vec3;
use itertools::Itertools;
use obj::Obj;
use simple_error::bail;
use std::error::Error;

// Read an OBJ file from `path`, and construct a DCEL.
impl<V: Default + HasPosition, E: Default, F: Default + HasNormal> Douconel<V, E, F> {
    pub fn from_obj(
        path: &str,
    ) -> Result<(Self, BiHashMap<usize, VertID>, BiHashMap<usize, FaceID>), Box<dyn Error>> {
        let obj = Obj::load(path).unwrap().data;
        let mesh = obj.objects[0].groups[0].clone();

        let faces = mesh
            .polys
            .iter()
            .map(|w| vec![w.0[0].0, w.0[1].0, w.0[2].0])
            .collect_vec();

        let res = Self::from_faces(faces.clone());

        let vert_positions = obj.position;
        let face_normals = obj.normal;

        if let Ok((mut douconel, vertex_map, face_map)) = res {
            for (inp_vertex_id, inp_vertex_pos) in vert_positions.iter().enumerate() {
                let vert_id = vertex_map.get_by_left(&inp_vertex_id).copied().unwrap();
                if let Some(v) = douconel.verts.get_mut(vert_id) {
                    v.set_position(Vec3::new(
                        inp_vertex_pos[0],
                        inp_vertex_pos[1],
                        inp_vertex_pos[2],
                    ));
                }
            }

            for (inp_face_id, inp_face_normal) in face_normals.iter().enumerate() {
                let face_id = face_map.get_by_left(&inp_face_id).copied().unwrap();
                if let Some(f) = douconel.faces.get_mut(face_id) {
                    f.set_normal(Vec3::new(
                        inp_face_normal[0],
                        inp_face_normal[1],
                        inp_face_normal[2],
                    ));
                }
            }

            Ok((douconel, vertex_map, face_map))
        } else {
            bail!(res.err().unwrap())
        }
    }
}
