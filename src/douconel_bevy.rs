#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use crate::douconel::{Douconel, FaceID};
use crate::douconel_embedded::{HasNormal, HasPosition};
use bevy_render::{
    color::Color,
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use std::collections::HashMap;

/// From an embedded DCEL, construct a mesh object that can be rendered using the Bevy framework. Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<V: HasPosition, E, F: HasNormal> Douconel<V, E, F> {
    #[must_use]
    pub fn bevy(&self, color_map: &HashMap<FaceID, Color>) -> Mesh {
        let mut mesh_triangle_list = Mesh::new(PrimitiveTopology::TriangleList);
        let number_of_faces_times_three = self.nr_faces() * 3;
        let mut vertex_positions = Vec::with_capacity(number_of_faces_times_three);
        let mut vertex_normals = Vec::with_capacity(number_of_faces_times_three);
        let mut vertex_colors = Vec::with_capacity(number_of_faces_times_three);

        self.faces
            .keys()
            .flat_map(|face_id| {
                self.corners(face_id)
                    .into_iter()
                    .map(move |vertex_id| (face_id, vertex_id))
            })
            .for_each(|(face_id, vertex_id)| {
                vertex_positions.push(self.position(vertex_id));
                vertex_normals.push(self.vert_normal(vertex_id));
                vertex_colors.push(
                    color_map
                        .get(&face_id)
                        .unwrap_or(&Color::PINK)
                        .as_rgba_f32(),
                );
            });

        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
        mesh_triangle_list.set_indices(Some(Indices::U32(
            (0..u32::try_from(number_of_faces_times_three).expect("u32 bit overflow")).collect(),
        )));

        mesh_triangle_list
    }
}
