use crate::{douconel::Douconel, douconel_embedded::HasPosition};
use core::panic;
use slotmap::Key;
use std::collections::HashMap;

type BevyMesh = bevy::prelude::Mesh;
type BevyVec = bevy::math::Vec3;
type BevyColor = bevy::color::Color;

/// Construct a Bevy mesh object (one that can be rendered using Bevy).
/// Requires a `color_map` to assign colors to faces. If no color is assigned to a face, it will be black.
impl<VertID: Key, V: Default + HasPosition, EdgeID: Key, E: Default, FaceID: Key, F: Default> Douconel<VertID, V, EdgeID, E, FaceID, F> {
    #[must_use]
    pub fn bevy(&self, color_map: &HashMap<FaceID, [f32; 3]>) -> BevyMesh {
        let mut vertex_positions = vec![];
        let mut vertex_normals = vec![];
        let mut vertex_colors = vec![];
        let mut vertex_uvs = vec![];

        for face_id in self.faces.keys() {
            let mut corners = self.corners(face_id);

            match corners.len() {
                0 | 1 | 2 => panic!("Face {:?} has too few corners", face_id),
                3 => {
                    let triangle = [corners[0], corners[1], corners[2]];
                    for vertex_id in triangle {
                        let position = self.position(vertex_id);
                        vertex_positions.push(BevyVec::new(position.x as f32, position.y as f32, position.z as f32));
                        let normal = self.vert_normal(vertex_id);
                        vertex_normals.push(BevyVec::new(normal.x as f32, normal.y as f32, normal.z as f32));
                        let color = color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK);
                        let color_lrgb = BevyColor::srgb(color[0], color[1], color[2]).to_linear();
                        vertex_colors.push([color_lrgb.red, color_lrgb.green, color_lrgb.blue, 1.]);
                        vertex_uvs.push([0., 0.]);
                    }
                    continue;
                }
                4 => {
                    let quad = [corners[0], corners[1], corners[2], corners[3]];
                    // let d1 = (v0 - v2).norm();
                    // let d2 = (v1 - v3).norm();
                    // if d1 < d2 {
                    //     // Split along v0-v2
                    // } else {
                    //     // Split along v1-v3
                    // }

                    let d1 = (self.position(corners[0]) - self.position(corners[2])).norm();
                    let d2 = (self.position(corners[1]) - self.position(corners[3])).norm();
                    if d1 < d2 {
                        let triangle1 = [corners[0], corners[1], corners[2]];
                        let triangle2 = [corners[2], corners[3], corners[0]];
                        for vertex_id in triangle1 {
                            let position = self.position(vertex_id);
                            vertex_positions.push(BevyVec::new(position.x as f32, position.y as f32, position.z as f32));
                            let normal = self.vert_normal(vertex_id);
                            vertex_normals.push(BevyVec::new(normal.x as f32, normal.y as f32, normal.z as f32));
                            let color = color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK);
                            let color_lrgb = BevyColor::srgb(color[0], color[1], color[2]).to_linear();
                            vertex_colors.push([color_lrgb.red, color_lrgb.green, color_lrgb.blue, 1.]);
                            vertex_uvs.push([0., 0.]);
                        }
                        for vertex_id in triangle2 {
                            let position = self.position(vertex_id);
                            vertex_positions.push(BevyVec::new(position.x as f32, position.y as f32, position.z as f32));
                            let normal = self.vert_normal(vertex_id);
                            vertex_normals.push(BevyVec::new(normal.x as f32, normal.y as f32, normal.z as f32));
                            let color = color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK);
                            let color_lrgb = BevyColor::srgb(color[0], color[1], color[2]).to_linear();
                            vertex_colors.push([color_lrgb.red, color_lrgb.green, color_lrgb.blue, 1.]);
                            vertex_uvs.push([0., 0.]);
                        }
                    } else {
                        let triangle1 = [corners[0], corners[1], corners[3]];
                        let triangle2 = [corners[1], corners[2], corners[3]];
                        for vertex_id in triangle1 {
                            let position = self.position(vertex_id);
                            vertex_positions.push(BevyVec::new(position.x as f32, position.y as f32, position.z as f32));
                            let normal = self.vert_normal(vertex_id);
                            vertex_normals.push(BevyVec::new(normal.x as f32, normal.y as f32, normal.z as f32));
                            let color = color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK);
                            let color_lrgb = BevyColor::srgb(color[0], color[1], color[2]).to_linear();
                            vertex_colors.push([color_lrgb.red, color_lrgb.green, color_lrgb.blue, 1.]);
                            vertex_uvs.push([0., 0.]);
                        }
                        for vertex_id in triangle2 {
                            let position = self.position(vertex_id);
                            vertex_positions.push(BevyVec::new(position.x as f32, position.y as f32, position.z as f32));
                            let normal = self.vert_normal(vertex_id);
                            vertex_normals.push(BevyVec::new(normal.x as f32, normal.y as f32, normal.z as f32));
                            let color = color_map.get(&face_id).unwrap_or(&hutspot::color::BLACK);
                            let color_lrgb = BevyColor::srgb(color[0], color[1], color[2]).to_linear();
                            vertex_colors.push([color_lrgb.red, color_lrgb.green, color_lrgb.blue, 1.]);
                            vertex_uvs.push([0., 0.]);
                        }
                    }
                }
                _ => {
                    panic!("Face {:?} has too many corners", face_id);
                }
            }
        }

        BevyMesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_indices(bevy::render::mesh::Indices::U32((0..u32::try_from(vertex_positions.len()).unwrap()).collect()))
        .with_inserted_attribute(BevyMesh::ATTRIBUTE_POSITION, vertex_positions)
        .with_inserted_attribute(BevyMesh::ATTRIBUTE_NORMAL, vertex_normals)
        .with_inserted_attribute(BevyMesh::ATTRIBUTE_COLOR, vertex_colors)
        .with_inserted_attribute(BevyMesh::ATTRIBUTE_UV_0, vertex_uvs)
    }
}
