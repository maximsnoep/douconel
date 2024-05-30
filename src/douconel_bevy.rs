use crate::douconel::{Douconel, FaceID};
use crate::douconel_embedded::HasPosition;
use bevy_render::{
    color::Color,
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use std::collections::HashMap;

/// From an embedded DCEL, construct a mesh object that can be rendered using the Bevy framework. Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<V: HasPosition, E, F> Douconel<V, E, F> {
    #[must_use]
    pub fn bevy(&self, color_map: &HashMap<FaceID, Color>) -> Mesh {
        const DEFAULT_COLOR: Color = Color::BLACK;
        let mut vertex_positions = vec![];
        let mut vertex_normals = vec![];
        let mut vertex_colors = vec![];

        self.faces
            .keys()
            .flat_map(|face_id| {
                let corners = self.corners(face_id);

                println!("{:?}", corners);

                match corners.len() {
                    3 => {
                        let vertex_id1 = corners.get(0).unwrap().clone();
                        let vertex_id2 = corners.get(1).unwrap().clone();
                        let vertex_id3 = corners.get(2).unwrap().clone();
                        vec![
                            (face_id, vertex_id1),
                            (face_id, vertex_id2),
                            (face_id, vertex_id3),
                        ]
                    }
                    4 => {
                        let vertex_id1 = corners.get(0).unwrap().clone();
                        let vertex_id2 = corners.get(1).unwrap().clone();
                        let vertex_id3 = corners.get(2).unwrap().clone();
                        let vertex_id4 = corners.get(3).unwrap().clone();
                        vec![
                            (face_id, vertex_id1),
                            (face_id, vertex_id2),
                            (face_id, vertex_id3),
                            (face_id, vertex_id1),
                            (face_id, vertex_id3),
                            (face_id, vertex_id4),
                        ]
                    }
                    _ => panic!("Face with {} corners is not supported.", corners.len()),
                }
            })
            .for_each(|(face_id, vertex_id)| {
                vertex_positions.push(self.position(vertex_id));
                vertex_normals.push(self.vert_normal(vertex_id));
                vertex_colors.push(
                    color_map
                        .get(&face_id)
                        .unwrap_or(&DEFAULT_COLOR)
                        .as_rgba_f32(),
                );
            });

        let mut mesh_triangle_list = Mesh::new(PrimitiveTopology::TriangleList);
        mesh_triangle_list.set_indices(Some(Indices::U32(
            (0..u32::try_from(vertex_positions.len()).expect("u32 bit overflow")).collect(),
        )));
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);

        mesh_triangle_list
    }
}
