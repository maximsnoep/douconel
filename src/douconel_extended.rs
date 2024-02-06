use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use rand::distributions::weighted;
use simple_error::bail;
use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;

use crate::douconel::{Douconel, EdgeID, FaceID, VertID};
use crate::utils;

pub trait HasPosition {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

impl<V: HasPosition, E, F> Douconel<V, E, F> {
    // Get position of a given vertex.
    pub fn position(&self, id: VertID) -> Vec3 {
        self.verts[id].position()
    }
}

pub trait HasNormal {
    fn normal(&self) -> Vec3;
    fn set_normal(&mut self, normal: Vec3);
}

impl<V, E, F: HasNormal> Douconel<V, E, F> {
    // Get normal of a given face.
    pub fn normal(&self, id: FaceID) -> Vec3 {
        self.faces[id].normal()
    }

    // Get the average normals around a given vertex.
    pub fn vert_normal(&self, id: VertID) -> Vec3 {
        self.star(id)
            .iter()
            .map(|&face_id| self.normal(face_id))
            .sum::<Vec3>()
            .normalize()
    }

    // Get the average normal of a given edge.
    pub fn edge_normal(&self, id: EdgeID) -> Vec3 {
        self.edge_normal_offset(id, 0.5)
    }

    // Get the average normal of a given edge, with offset
    pub fn edge_normal_offset(&self, edge_id: EdgeID, offset: f32) -> Vec3 {
        let (f1, f2) = self.faces(edge_id);
        (self.normal(f1) * (offset) + self.normal(f2) * (1. - offset)).normalize()
    }
}

pub trait HasColor {
    fn color(&self) -> Color;
    fn set_color(&mut self, color: Color);
}

impl<V, E, F: HasColor> Douconel<V, E, F> {
    // Get color of a given face.
    pub fn color(&self, id: FaceID) -> Color {
        self.faces[id].color()
    }
}

// Read an STL file from `path`, and construct a DCEL.
impl<V: Default + HasPosition, E: Default, F: Default + HasNormal + HasColor> Douconel<V, E, F> {
    pub fn from_stl(path: &str) -> Result<Self, Box<dyn Error>> {
        let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

        let faces = stl.faces.iter().map(|f| f.vertices.to_vec()).collect_vec();

        if let Ok((mut douconel, vertex_map, face_map)) = Self::from_faces(faces) {
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
                    f.set_color(Color::BLACK);
                }
            }

            Ok(douconel)
        } else {
            bail!("Failed to construct douconel");
        }
    }
}

// Implement helper functions for when vertices have a defined position
impl<V: HasPosition, E, F> Douconel<V, E, F> {
    // Get centroid of a given face. Be careful with concave faces, the centroid might lay outside the face.
    pub fn centroid(&self, face_id: FaceID) -> Vec3 {
        let mut centroid = Vec3::ZERO;
        let mut count = 0;
        for edge_id in self.edges(face_id) {
            centroid += self.position(self.root(edge_id));
            count += 1;
        }
        centroid / count as f32
    }

    // Get midpoint of a given edge.
    pub fn midpoint(&self, edge_id: EdgeID) -> Vec3 {
        self.midpoint_offset(edge_id, 0.5)
    }

    // Get midpoint of a given edge with some offset
    pub fn midpoint_offset(&self, edge_id: EdgeID, offset: f32) -> Vec3 {
        self.position(self.root(edge_id)) + self.vector(edge_id) * offset
    }

    // Get vector of a given edge.
    pub fn vector(&self, id: EdgeID) -> Vec3 {
        let (u, v) = self.endpoints(id);
        self.position(v) - self.position(u)
    }

    // Get length of a given edge.
    pub fn length(&self, id: EdgeID) -> f32 {
        self.vector(id).length()
    }

    // Get distance between two vertices.
    pub fn distance(&self, v_a: VertID, v_b: VertID) -> f32 {
        self.position(v_a).distance(self.position(v_b))
    }

    // Get angle between two edges.
    pub fn angle(&self, e_a: EdgeID, e_b: EdgeID) -> f32 {
        self.vector(e_a).angle_between(self.vector(e_b))
    }

    // Get angular defect of a vertex (2pi minus the sum of all the angles at the vertex).
    pub fn defect(&self, id: VertID) -> f32 {
        let mut sum_of_angles = 0.;
        let outgoing_edges = self.outgoing(id);
        for outgoing_edge_id in outgoing_edges {
            let incoming_edge_id = self.twin(outgoing_edge_id);
            let next_edge_id = self.next(incoming_edge_id);
            let angle = self.angle(outgoing_edge_id, next_edge_id);
            sum_of_angles += angle;
        }
        2. * std::f32::consts::PI - sum_of_angles
    }

    // To petgraph, directed graph, based on the DCEL, with weights based on Euclidean distance.
    pub fn graph(&self) -> DiGraphMap<VertID, f32> {
        let mut edges = vec![];
        for id in self.edges.keys() {
            edges.push((self.root(id), self.root(self.twin(id)), self.length(id)));
        }

        DiGraphMap::<VertID, f32>::from_edges(edges)
    }

    // To petgraph: dual graph
    pub fn dual_graph(&self) -> DiGraphMap<FaceID, f32> {
        let mut edges = vec![];
        for id in self.faces.keys() {
            for n_id in self.fneighbors(id) {
                edges.push((id, n_id, self.centroid(id).distance(self.centroid(n_id))));
            }
        }

        DiGraphMap::<FaceID, f32>::from_edges(edges)
    }
}

impl<V: HasPosition, E, F: HasNormal> Douconel<V, E, F> {
    // To petgraph: edge graph with <>DWAJD@$@!KM# edge weights
    pub fn edge_graph(&self, direction: Vec3, gamma: f32, filter: f32) -> DiGraphMap<EdgeID, f32> {
        let mut edges = vec![];
        let mut verts = HashSet::new();

        for id in self.edges.keys() {
            for n_id in self.edges(self.face(id)) {
                if id == n_id {
                    continue;
                }

                let edge_direction = (self.midpoint(n_id) - self.midpoint(id)).normalize();
                let edge_normal = self.edge_normal(id);
                let cross = edge_direction.cross(edge_normal);
                let angle = (direction.angle_between(cross) / std::f32::consts::PI) * 180.;
                let weight = angle.powf(gamma);

                if angle < filter {
                    edges.push((id, n_id, weight));
                    verts.insert(id);
                    verts.insert(n_id);
                }
            }
        }

        for id in verts {
            let n_id = self.twin(id);

            let edge_direction = self.vector(n_id).normalize();
            let angle = (direction.angle_between(edge_direction) / std::f32::consts::PI) * 180.;
            println!("{:?}", angle);
            let weight = angle.powf(gamma);

            if angle < filter {
                edges.push((id, n_id, weight));
            }
        }

        DiGraphMap::<EdgeID, f32>::from_edges(edges)
    }
}

// Construct a mesh object that can be rendered using the Bevy framework.
impl<V: HasPosition, E, F: HasNormal + HasColor> Douconel<V, E, F> {
    pub fn bevy(&self) -> Mesh {
        let mut mesh_triangle_list = Mesh::new(PrimitiveTopology::TriangleList);
        let number_of_faces = self.nr_faces();
        let mut vertex_positions = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_normals = Vec::with_capacity(number_of_faces * 3);
        let mut vertex_colors = Vec::with_capacity(number_of_faces * 3);

        for face_id in self.faces.keys() {
            let color = self.color(face_id).as_rgba_f32();

            for vertex_id in self.corners(face_id) {
                let position = self.position(vertex_id);
                vertex_positions.push(position);
                let normal = utils::average(
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
