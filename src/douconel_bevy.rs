type Mesh = bevy_render::mesh::Mesh;
type Color = bevy_render::color::Color;

/// Construct a Bevy mesh object (one that can be rendered using Bevy).
/// Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<V: Default + crate::douconel_embedded::HasPosition, E: Default, F: Default>
    crate::douconel::Douconel<V, E, F>
{
    #[must_use]
    pub fn bevy(
        &self,
        color_map: &std::collections::HashMap<crate::douconel::FaceID, Color>,
    ) -> Mesh {
        const DEFAULT_COLOR: Color = Color::BLACK;
        let mut vertex_positions = vec![];
        let mut vertex_normals = vec![];
        let mut vertex_colors = vec![];

        for face_id in self.faces.keys() {
            let mut corners = self.corners(face_id);

            'outer: while corners.len() > 2 {
                let cur_corners = corners.clone();
                for i in 0..corners.len() {
                    let j = (i + 1) % corners.len();
                    let k = (i + 2) % corners.len();

                    let u = self.position(corners[i]) - self.position(corners[j]);
                    let v = self.position(corners[k]) - self.position(corners[j]);
                    let n = self.normal(face_id);

                    if u.cross(v).dot(n) < 0.
                        && cur_corners.iter().all(|&corner| {
                            corner == corners[i]
                                || corner == corners[j]
                                || corner == corners[k]
                                || !potpoursi::math::inside_triangle(
                                    self.position(corner),
                                    self.position(corners[i]),
                                    self.position(corners[j]),
                                    self.position(corners[k]),
                                )
                        })
                    {
                        let triangle = [corners[i], corners[j], corners[k]];
                        for vertex_id in triangle {
                            vertex_positions.push(self.position(vertex_id));
                            vertex_normals.push(self.vert_normal(vertex_id));
                            vertex_colors.push(
                                color_map
                                    .get(&face_id)
                                    .unwrap_or(&DEFAULT_COLOR)
                                    .as_rgba_f32(),
                            );
                        }

                        corners.remove(j);
                        continue 'outer;
                    }
                }
            }

            // for vertex_id in triangles.into_iter().flatten() {}
        }

        Mesh::new(bevy_render::render_resource::PrimitiveTopology::TriangleList)
            .with_indices(Some(bevy_render::mesh::Indices::U32(
                (0..u32::try_from(vertex_positions.len()).unwrap()).collect(),
            )))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors)
    }
}
