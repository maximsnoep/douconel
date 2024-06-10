type BevyMesh = bevy::prelude::Mesh;
type BevyVec = bevy::math::Vec3;

/// Construct a Bevy mesh object (one that can be rendered using Bevy).
/// Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<V: Default + crate::douconel_embedded::HasPosition, E: Default, F: Default>
    crate::douconel::Douconel<V, E, F>
{
    #[must_use]
    pub fn bevy(
        &self,
        color_map: &std::collections::HashMap<crate::douconel::FaceID, [f32; 3]>,
    ) -> BevyMesh {
        let mut vertex_positions = vec![];
        let mut vertex_normals = vec![];
        let mut vertex_colors = vec![];

        for face_id in self.faces.keys() {
            let mut corners = self.corners(face_id);

            'outer: while corners.len() > 2 {
                for i in 0..corners.len() {
                    let j = (i + 1) % corners.len();
                    let k = (i + 2) % corners.len();

                    let a = self.position(corners[i]);
                    let b = self.position(corners[j]);
                    let c = self.position(corners[k]);
                    let n = self.normal(face_id);

                    if hutspot::geom::calculate_orientation(a, b, c, n)
                        == hutspot::geom::Orientation::CCW
                        && corners
                            .clone()
                            .into_iter()
                            .filter(|&corner| {
                                corner != corners[i] && corner != corners[j] && corner != corners[k]
                            })
                            .all(|corner| {
                                !hutspot::geom::is_point_inside_triangle(
                                    self.position(corner),
                                    (
                                        self.position(corners[i]),
                                        self.position(corners[j]),
                                        self.position(corners[k]),
                                    ),
                                )
                            })
                    {
                        let triangle = [corners[i], corners[j], corners[k]];
                        for vertex_id in triangle {
                            let position = self.position(vertex_id);
                            vertex_positions.push(BevyVec::new(
                                position.x as f32,
                                position.y as f32,
                                position.z as f32,
                            ));
                            let normal = self.vert_normal(vertex_id);
                            vertex_normals.push(BevyVec::new(
                                normal.x as f32,
                                normal.y as f32,
                                normal.z as f32,
                            ));
                            let color = color_map.get(&face_id).unwrap_or(&[0., 0., 0.]);
                            vertex_colors.push([color[0], color[1], color[2], 1.]);
                        }

                        corners.remove(j);
                        continue 'outer;
                    }
                }
            }

            // for vertex_id in triangles.into_iter().flatten() {}
        }

        BevyMesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
            .with_indices(Some(bevy::render::mesh::Indices::U32(
                (0..u32::try_from(vertex_positions.len()).unwrap()).collect(),
            )))
            .with_inserted_attribute(BevyMesh::ATTRIBUTE_POSITION, vertex_positions)
            .with_inserted_attribute(BevyMesh::ATTRIBUTE_NORMAL, vertex_normals)
            .with_inserted_attribute(BevyMesh::ATTRIBUTE_COLOR, vertex_colors)
    }
}
