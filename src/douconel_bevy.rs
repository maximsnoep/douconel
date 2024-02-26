use crate::douconel::{Douconel, FaceID};
use crate::douconel_embedded::{HasNormal, HasPosition};
use bevy_render::{
    color::Color,
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use std::collections::HashMap;

/// From an embedded DCEL, construct a mesh object that can be rendered using the Bevy framework. Requires a color_map to assign colors to faces. If no color is assigned to a face, it will be black.
impl<V: HasPosition, E, F: HasNormal> Douconel<V, E, F> {
    pub fn bevy(&self, color_map: HashMap<FaceID, Color>) -> Mesh {
        let mut mesh_triangle_list = Mesh::new(PrimitiveTopology::TriangleList);
        let number_of_faces = self.nr_faces();
        let mut vertex_positions = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_normals = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_colors = Vec::with_capacity(number_of_faces * 3);

        for face_id in self.faces.keys() {
            let color = match color_map.get(&face_id) {
                Some(color) => color.to_owned(),
                None => Color::BLACK,
            }
            .as_rgba_f32();

            for vertex_id in self.corners(face_id) {
                let position = self.position(vertex_id);
                vertex_positions.push(position);
                let normal = potpoursi::math::average(
                    self.star(vertex_id)
                        .iter()
                        .map(|&face_id| self.normal(face_id)),
                );

                vertex_normals.push(normal);
                vertex_colors.push(color);
            }
        }

        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
        mesh_triangle_list.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
        mesh_triangle_list.set_indices(Some(Indices::U32(
            (0..number_of_faces as u32 * 3).collect(),
        )));

        mesh_triangle_list
    }
}
