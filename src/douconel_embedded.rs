#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use crate::douconel::{Douconel, EdgeID, FaceID, FaceMap, VertID, VertMap};
use glam::Vec3;
use itertools::Itertools;
use obj::Obj;
use ordered_float::OrderedFloat;
use rayon::vec;
use simple_error::bail;
use std::{error::Error, fs::OpenOptions};

pub trait HasPosition {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

pub trait HasNormal {
    fn normal(&self) -> Vec3;
    fn set_normal(&mut self, normal: Vec3);
}

// Embedded vertices (have a position)
#[derive(Default, Copy, Clone, Debug)]
pub struct EmbeddedVertex {
    position: Vec3,
}

impl HasPosition for EmbeddedVertex {
    fn position(&self) -> Vec3 {
        self.position
    }
    fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}

impl<V: HasPosition, E, F> Douconel<V, E, F> {
    // Get position of a given vertex.
    #[must_use]
    pub fn position(&self, id: VertID) -> Vec3 {
        self.verts
            .get(id)
            .unwrap_or_else(|| panic!("V:{id:?} not initialized"))
            .position()
    }

    // Get centroid of a given polygonal face.
    // https://en.wikipedia.org/wiki/Centroid
    // Be careful with concave faces, the centroid might lay outside the face.
    #[must_use]
    pub fn centroid(&self, face_id: FaceID) -> Vec3 {
        potpoursi::math::average(
            self.edges(face_id)
                .iter()
                .map(|&edge_id| self.position(self.root(edge_id))),
        )
    }

    // Get midpoint of a given edge.
    #[must_use]
    pub fn midpoint(&self, edge_id: EdgeID) -> Vec3 {
        self.midpoint_offset(edge_id, 0.5)
    }

    // Get midpoint of a given edge with some offset
    #[must_use]
    pub fn midpoint_offset(&self, edge_id: EdgeID, offset: f32) -> Vec3 {
        self.position(self.root(edge_id)) + self.vector(edge_id) * offset
    }

    // Get vector of a given edge.
    #[must_use]
    pub fn vector(&self, id: EdgeID) -> Vec3 {
        let (u, v) = self.endpoints(id);
        self.position(v) - self.position(u)
    }

    // Get length of a given edge.
    #[must_use]
    pub fn length(&self, id: EdgeID) -> f32 {
        self.vector(id).length()
    }

    // Get distance between two vertices.
    #[must_use]
    pub fn distance(&self, v_a: VertID, v_b: VertID) -> f32 {
        self.position(v_a).distance(self.position(v_b))
    }

    // Get angle (in radians) between two edges.
    #[must_use]
    pub fn angle(&self, e_a: EdgeID, e_b: EdgeID) -> f32 {
        self.vector(e_a).angle_between(self.vector(e_b))
    }

    // Get angular defect of a vertex (2PI - C, where C = the sum of all the angles at the vertex).
    // See https://en.wikipedia.org/wiki/Angular_defect
    #[must_use]
    pub fn defect(&self, id: VertID) -> f32 {
        let sum_of_angles = self.outgoing(id).iter().fold(0., |sum, &outgoing_edge_id| {
            let incoming_edge_id = self.twin(outgoing_edge_id);
            let next_edge_id = self.next(incoming_edge_id);
            let angle = self.angle(outgoing_edge_id, next_edge_id);
            sum + angle
        });

        // 2PI - C
        2.0f32.mul_add(std::f32::consts::PI, -sum_of_angles)
    }
}

// Embedded faces (have a normal)
#[derive(Default, Copy, Clone, Debug)]
pub struct EmbeddedFace {
    normal: Vec3,
}

impl HasNormal for EmbeddedFace {
    fn normal(&self) -> Vec3 {
        self.normal
    }
    fn set_normal(&mut self, normal: Vec3) {
        self.normal = normal;
    }
}

impl<V, E, F: HasNormal> Douconel<V, E, F> {
    // Get normal of a given face.
    #[must_use]
    pub fn normal(&self, id: FaceID) -> Vec3 {
        self.faces
            .get(id)
            .unwrap_or_else(|| panic!("F:{id:?} not initialized"))
            .normal()
    }

    // Get the average normals around a given vertex.
    #[must_use]
    pub fn vert_normal(&self, id: VertID) -> Vec3 {
        self.star(id)
            .iter()
            .map(|&face_id| self.normal(face_id))
            .sum::<Vec3>()
            .normalize()
    }

    // Get the average normal of a given edge.
    #[must_use]
    pub fn edge_normal(&self, id: EdgeID) -> Vec3 {
        self.edge_normal_offset(id, 0.5)
    }

    // Get the average normal of a given edge, with offset
    #[must_use]
    pub fn edge_normal_offset(&self, edge_id: EdgeID, offset: f32) -> Vec3 {
        let [f1, f2] = self.faces(edge_id);
        (self.normal(f1) * (offset) + self.normal(f2) * (1. - offset)).normalize()
    }
}

// Construct a DCEL with faces.
// Embed vertices using positions.
// Embed faces using normals.
impl<V: Default + HasPosition, E: Default, F: Default + HasNormal> Douconel<V, E, F> {
    pub fn from_embedded_faces(
        faces: &[Vec<usize>],
        vertex_positions: &[Vec3],
        face_normals: &[Vec3],
    ) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
        let non_embedded = Self::from_faces(faces);
        if let Ok((mut douconel, vertex_map, face_map)) = non_embedded {
            for (inp_vertex_id, inp_vertex_position) in vertex_positions.iter().copied().enumerate()
            {
                let vertex_id = vertex_map
                    .get_by_left(&inp_vertex_id)
                    .copied()
                    .unwrap_or_else(|| panic!("V:{inp_vertex_id} not initialized"));
                douconel
                    .verts
                    .get_mut(vertex_id)
                    .unwrap_or_else(|| panic!("V:{vertex_id:?} not initialized"))
                    .set_position(inp_vertex_position);
            }
            for (inp_face_id, inp_face_normal) in face_normals.iter().copied().enumerate() {
                let face_id = face_map
                    .get_by_left(&inp_face_id)
                    .copied()
                    .unwrap_or_else(|| panic!("F:{inp_face_id} not initialized"));
                douconel
                    .faces
                    .get_mut(face_id)
                    .unwrap_or_else(|| panic!("F:{face_id:?} not initialized"))
                    .set_normal(inp_face_normal);
            }

            Ok((douconel, vertex_map, face_map))
        } else {
            bail!(non_embedded.err().unwrap())
        }
    }

    // Read an STL file from `path`, and construct an embedded DCEL.
    pub fn from_stl(path: &str) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
        let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

        let faces = stl.faces.iter().map(|f| f.vertices.to_vec()).collect_vec();

        let vertex_positions = stl
            .vertices
            .iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]))
            .collect_vec();
        let face_normals = stl
            .faces
            .iter()
            .map(|f| Vec3::new(f.normal[0], f.normal[1], f.normal[2]))
            .collect_vec();

        Self::from_embedded_faces(&faces, &vertex_positions, &face_normals)
    }

    // Read an OBJ file from `path`, and construct an embedded DCEL.
    pub fn from_obj(path: &str) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
        let obj = Obj::load(path)?.data;
        let mesh = obj.objects[0].groups[0].clone();

        let faces = mesh
            .polys
            .iter()
            .map(|w| vec![w.0[0].0, w.0[1].0, w.0[2].0])
            .collect_vec();

        let vertex_positions = obj
            .position
            .iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]))
            .collect_vec();
        let face_normals = obj
            .normal
            .iter()
            .map(|n| Vec3::new(n[0], n[1], n[2]))
            .collect_vec();

        Self::from_embedded_faces(&faces, &vertex_positions, &face_normals)
    }

    // Weight function
    pub fn weight_function_euclidean(&self) -> impl Fn(VertID, VertID) -> OrderedFloat<f32> + '_ {
        |a, b| ordered_float::OrderedFloat(self.position(a).distance(self.position(b)))
    }

    // Weight function
    pub fn weight_function_angle_edges(
        &self,
        slack: i32,
    ) -> impl Fn(EdgeID, EdgeID) -> OrderedFloat<f32> + '_ {
        move |a, b| {
            ordered_float::OrderedFloat((self.vector(a).angle_between(self.vector(b))).powi(slack))
        }
    }

    // Weight function
    pub fn weight_function_angle_edgepairs(
        &self,
        slack: i32,
    ) -> impl Fn((EdgeID, EdgeID), (EdgeID, EdgeID)) -> OrderedFloat<f32> + '_ {
        move |a, b| {
            let vector_a = self.midpoint(a.1) - self.midpoint(a.0);
            let vector_b = self.midpoint(b.1) - self.midpoint(b.0);
            ordered_float::OrderedFloat((vector_a.angle_between(vector_b)).powi(slack))
        }
    }

    // Weight function
    pub fn weight_function_angle_edgepairs_aligned(
        &self,
        angular_slack: i32,
        alignment_slack: i32,
        axis: Vec3,
    ) -> impl Fn((EdgeID, EdgeID), (EdgeID, EdgeID)) -> OrderedFloat<f32> + '_ {
        move |a, b| {
            let vector_a = self.midpoint(a.1) - self.midpoint(a.0);
            let vector_b = self.midpoint(b.1) - self.midpoint(b.0);
            let normal_a = self.edge_normal(a.0);
            let normal_b = self.edge_normal(b.0);

            let cross_a = vector_a.cross(normal_a);
            let cross_b = vector_b.cross(normal_b);

            let angle_a = cross_a.angle_between(axis);
            let angle_b = cross_b.angle_between(axis);

            let angle_ab = vector_a.angle_between(vector_b);
            let weight = angle_ab.powi(angular_slack)
                + (angle_a).powi(alignment_slack)
                + (angle_b).powi(alignment_slack);

            ordered_float::OrderedFloat(weight)
        }
    }
}
