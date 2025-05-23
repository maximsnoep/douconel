use crate::{douconel::Douconel, douconel_embedded::HasPosition};
use bevy::{
    asset::RenderAssetUsages,
    color::{Color, ColorToComponents},
    gizmos::GizmoAsset,
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use core::panic;
use hutspot::{draw::DrawableLine, geom::Vector3D};
use slotmap::Key;
use std::collections::HashMap;

#[derive(Default)]
pub struct BevyMeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
}

impl BevyMeshBuilder {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            normals: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            uvs: Vec::with_capacity(capacity),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub fn add_vertex(&mut self, position: &Vector3D, normal: &Vector3D, color: &hutspot::color::Color) {
        self.positions.push(Vec3::new(position.x as f32, position.y as f32, position.z as f32));
        self.normals.push(Vec3::new(normal.x as f32, normal.y as f32, normal.z as f32));
        self.colors.push(Color::srgb(color[0], color[1], color[2]).to_linear().to_f32_array());
        self.uvs.push([0., 0.]);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn normalize(&mut self, scale: f64, translation: Vector3D) {
        for position in &mut self.positions {
            *position = Vec3::new(
                position.x.mul_add(scale as f32, translation.x as f32),
                position.y.mul_add(scale as f32, translation.y as f32),
                position.z.mul_add(scale as f32, translation.z as f32),
            );
        }
    }

    #[must_use]
    #[inline]
    pub fn build(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
            .with_inserted_indices(Indices::U32((0..u32::try_from(self.positions.len()).unwrap()).collect()))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
    }
}

/// Construct a Bevy mesh object (one that can be rendered using Bevy).
/// Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<VertID: Key, V: Default + HasPosition, EdgeID: Key, E: Default, FaceID: Key, F: Default> Douconel<VertID, V, EdgeID, E, FaceID, F> {
    #[must_use]
    pub fn bevy(&self, color_map: &HashMap<FaceID, [f32; 3]>) -> (Mesh, Vector3D, f64) {
        if self.faces.is_empty() {
            return (BevyMeshBuilder::with_capacity(0).build(), Vector3D::new(0., 0., 0.), 1.);
        }

        let k = self.corners(self.faces.keys().next().unwrap()).len();

        let mut bevy_mesh_builder = BevyMeshBuilder::with_capacity(self.faces.len() * (k - 2) * 3);

        for face_id in self.faces.keys() {
            let corners = self.corners(face_id);

            match corners.len() {
                0..=2 => panic!("Face {:?} has too few corners", face_id),
                3 => {
                    let triangle = [corners[0], corners[1], corners[2]];
                    for vertex_id in triangle {
                        bevy_mesh_builder.add_vertex(
                            &self.position(vertex_id),
                            &self.vert_normal(vertex_id),
                            color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK),
                        );
                    }
                }
                4 => {
                    let d1 = (self.position(corners[0]) - self.position(corners[2])).norm();
                    let d2 = (self.position(corners[1]) - self.position(corners[3])).norm();
                    let triangle = {
                        if d1 < d2 {
                            [corners[0], corners[1], corners[2], corners[2], corners[3], corners[0]]
                        } else {
                            [corners[0], corners[1], corners[3], corners[1], corners[2], corners[3]]
                        }
                    };
                    for vertex_id in triangle {
                        bevy_mesh_builder.add_vertex(
                            &self.position(vertex_id),
                            &self.vert_normal(vertex_id),
                            color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK),
                        );
                    }
                }
                _ => {
                    // not implemented yet
                    unimplemented!("Face {:?} has degree more than 4 ({})", face_id, corners.len());
                }
            }
        }

        let (scale, translation) = self.scale_translation();
        bevy_mesh_builder.normalize(scale, translation);
        let mesh = bevy_mesh_builder.build();
        (mesh, translation, scale)
    }

    // Construct a Bevy gizmos object of the wireframe (one that can be rendered using Bevy)
    #[must_use]
    pub fn gizmos(&self, color: [f32; 3]) -> GizmoAsset {
        let mut gizmo = GizmoAsset::new();
        let (scale, translation) = self.scale_translation();

        for &(u, v) in &self.edges_positions() {
            let line: DrawableLine = DrawableLine::from_line(u, v, Vector3D::new(0., 0., 0.), translation, scale);
            let c = Color::srgb(color[0], color[1], color[2]);
            gizmo.line(line.u, line.v, c);
        }

        gizmo
    }

    #[must_use]
    pub fn scale_translation(&self) -> (f64, Vector3D) {
        let scale = self.scale();
        let (center, _half_extents) = self.get_aabb();
        (scale, -scale * center)
    }

    #[must_use]
    pub fn scale(&self) -> f64 {
        let (_, half_extents) = self.get_aabb();
        20. * (1. / half_extents.max())
    }
}
