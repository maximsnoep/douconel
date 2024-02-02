use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use simple_error::bail;
use std::error::Error;
use std::fs::OpenOptions;

use crate::douconel::{Douconel, EdgeID, ElemID, FaceID, VertID};
use crate::utils;

pub trait HasPosition {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

pub trait HasNormal {
    fn normal(&self) -> Vec3;
    fn set_normal(&mut self, normal: Vec3);
}

pub trait HasColor {
    fn color(&self) -> Color;
    fn set_color(&mut self, color: Color);
}

// Implement from_stl for Douconel with HasPosition on vertices and HasNormal on faces
impl<
        V: Default + Copy + Clone + HasPosition,
        E: Default + Copy + Clone,
        F: Default + Copy + Clone + HasNormal + HasColor,
    > Douconel<V, E, F>
{
    // Read an STL file from `path`, and construct a DCEL.
    pub fn from_stl(path: &str) -> Result<Self, Box<dyn Error>> {
        let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

        let faces = stl.faces.iter().map(|f| f.vertices.to_vec()).collect_vec();

        if let Ok((mut douconel, vertex_map, face_map)) = Self::from_faces(faces) {
            for (inp_vertex_id, inp_vertex_pos) in stl.vertices.iter().enumerate() {
                let vert_id = vertex_map[&inp_vertex_id];
                if let Some(vert) = douconel.verts.get_mut(vert_id) {
                    vert.aux.set_position(Vec3::new(
                        inp_vertex_pos[0],
                        inp_vertex_pos[1],
                        inp_vertex_pos[2],
                    ));
                }
            }
            for (inp_face_id, inp_face_normal) in stl.faces.iter().enumerate() {
                let face_id = face_map[&inp_face_id];
                if let Some(face) = douconel.faces.get_mut(face_id) {
                    face.aux.set_normal(Vec3::new(
                        inp_face_normal.normal[0],
                        inp_face_normal.normal[1],
                        inp_face_normal.normal[2],
                    ));

                    face.aux.set_color(Color::BLACK);
                }
            }

            Ok(douconel)
        } else {
            bail!("Failed to construct douconel");
        }
    }
}

// Implement helper functions for when vertices have a defined position
impl<
        V: Default + Copy + Clone + HasPosition,
        E: Default + Copy + Clone,
        F: Default + Copy + Clone,
    > Douconel<V, E, F>
{
    // Get position of a given vertex.
    pub fn position(&self, id: VertID) -> Option<Vec3> {
        Some(self.verts.get(id)?.aux.position())
    }

    // Get centroid of a given face. Be careful with concave faces, the centroid might lay outside the face.
    pub fn centroid(&self, face_id: FaceID) -> Option<Vec3> {
        let mut centroid = Vec3::ZERO;
        let mut count = 0;
        for edge_id in self.edges(ElemID::Face(face_id))? {
            centroid += self.position(self.root(edge_id)?)?;
            count += 1;
        }
        Some(centroid / count as f32)
    }

    // Get vector of a given edge.
    pub fn vector(&self, id: EdgeID) -> Option<Vec3> {
        let ends = self.endpoints(id)?;
        Some(self.position(ends[1])? - self.position(ends[0])?)
    }

    // Get length of a given edge.
    pub fn length(&self, id: EdgeID) -> Option<f32> {
        Some(self.vector(id)?.length())
    }

    // Get distance between two vertices.
    pub fn distance(&self, v_a: VertID, v_b: VertID) -> Option<f32> {
        Some(self.position(v_a)?.distance(self.position(v_b)?))
    }

    // Get angle between two edges.
    pub fn angle(&self, e_a: EdgeID, e_b: EdgeID) -> Option<f32> {
        Some(self.vector(e_a)?.angle_between(self.vector(e_b)?))
    }

    // Get angular defect of a vertex (2pi minus the sum of all the angles at the vertex).
    pub fn defect(&self, id: VertID) -> Option<f32> {
        let mut sum_of_angles = 0.;
        let outgoing_edges = self.edges(ElemID::Vert(id))?;
        for outgoing_edge_id in outgoing_edges {
            let incoming_edge_id = self.twin(outgoing_edge_id)?;
            let next_edge_id = self.next(incoming_edge_id)?;
            let angle = self.angle(outgoing_edge_id, next_edge_id)?;
            sum_of_angles += angle;
        }
        Some(2. * std::f32::consts::PI - sum_of_angles)
    }

    pub fn petgraph(&self) -> Option<DiGraphMap<VertID, f32>> {
        let mut edges = vec![];
        for id in self.edges.keys() {
            edges.push((
                self.root(id).unwrap(),
                self.root(self.twin(id).unwrap()).unwrap(),
                self.length(id).unwrap(),
            ));
        }

        Some(DiGraphMap::<VertID, f32>::from_edges(edges))
    }
}

impl<V: Clone, E: Clone, F: Clone + HasNormal> Douconel<V, E, F> {
    // Get color of a given face.
    pub fn normal(&self, id: FaceID) -> Option<Vec3> {
        Some(self.faces.get(id)?.clone().aux.normal())
    }
}

impl<
        V: Default + Copy + Clone,
        E: Default + Copy + Clone,
        F: Default + Copy + Clone + HasColor,
    > Douconel<V, E, F>
{
    // Get color of a given face.
    pub fn color(&self, id: FaceID) -> Option<Color> {
        Some(self.faces.get(id)?.clone().aux.color())
    }
}

impl<
        V: Default + Copy + Clone + HasPosition,
        E: Default + Copy + Clone,
        F: Default + Copy + Clone + HasNormal + HasColor,
    > Douconel<V, E, F>
{
    // Construct a mesh object that can be rendered using the Bevy framework.
    pub fn bevy(&self) -> Option<Mesh> {
        let mut mesh_triangle_list = Mesh::new(PrimitiveTopology::TriangleList);
        let number_of_faces = self.nr_faces();
        let mut vertex_positions = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_normals = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_colors = Vec::with_capacity(number_of_faces * 3);

        for face_id in self.faces.keys() {
            let color = match self.color(face_id) {
                Some(c) => c.as_rgba_f32(),
                None => Color::PINK.as_rgba_f32(),
            };

            for vertex_id in self.vertices(ElemID::Face(face_id))? {
                let position = self.position(vertex_id)?;
                vertex_positions.push(position);
                let normal = utils::average(
                    self.faces(ElemID::Vert(vertex_id))?
                        .iter()
                        .map(|&face_id| self.normal(face_id).unwrap()),
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

        Some(mesh_triangle_list)
    }
}
