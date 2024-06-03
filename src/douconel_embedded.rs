use crate::douconel::{Douconel, EdgeID, FaceID, FaceMap, VertID, VertMap};
use glam::Vec3;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use simple_error::bail;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
};

pub trait HasPosition {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
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

impl<V: Default + HasPosition, E: Default, F: Default> Douconel<V, E, F> {
    // This is a struct that defines an embedded mesh with vertices (with position), edges, and faces (with clockwise ordering).
    // This embedded mesh is:
    //      a closed 2-manifold: Each edge corresponds to exactly two faces.
    //      connected: There exists a path between any two vertices.
    //      orientable: There exists a consistent normal for each face.
    //      polygonal: Each face is a simple polygon (lies in a plane, no intersections).
    // These requirements will be true per construction.
    pub fn from_embedded_faces(
        faces: &[Vec<usize>],
        vertex_positions: &[Vec3],
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

            // Make sure the mesh is polygonal
            for face_id in douconel.faces.keys() {
                // Check that the face is planar
                let face_normal = douconel.normal(face_id);
                for edge_id in douconel.edges(face_id) {
                    let u = douconel.vector(douconel.twin(edge_id));
                    let v = douconel.vector(douconel.next(edge_id));
                    if u.cross(v).dot(face_normal) < 0. {
                        bail!("Face {face_id:?} is not planar.");
                    }
                }

                // Check that the face is simple
                for edge_a in douconel.edges(face_id) {
                    for edge_b in douconel.edges(face_id) {
                        if edge_a == edge_b {
                            continue;
                        }
                        if potpoursi::math::intersection(
                            douconel.vector(edge_a),
                            douconel.vector(edge_b),
                        ) {
                            bail!("Face {face_id:?} has intersecting edges.");
                        }
                    }
                }
            }

            Ok((douconel, vertex_map, face_map))
        } else {
            bail!(non_embedded.err().unwrap())
        }
    }

    // Read an STL file from `path`, and construct an embedded DCEL.
    // Todo: Write own parser, to avoid dependency on stl_io.
    pub fn from_stl(path: &str) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
        let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

        let faces = stl.faces.iter().map(|f| f.vertices.to_vec()).collect_vec();

        let verts = stl
            .vertices
            .iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]))
            .collect_vec();

        Self::from_embedded_faces(&faces, &verts)
    }

    // Read an OBJ file from `path`, and construct an embedded DCEL.
    pub fn from_obj(path: &str) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
        // Load the obj file

        // go through all lines of `path`
        // for each line, check if it starts with "v" or "f"
        // if it starts with "v", parse the line as a vertex
        // if it starts with "f", parse the line as a face
        // if it starts with anything else, ignore the line
        // after going through all lines, construct the DCEL
        let mut verts = vec![];
        let mut faces = vec![];
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("v ") {
                let vertex_position = line.split_whitespace().skip(1).collect_vec();
                let x = vertex_position[0].parse::<f32>()?;
                let y = vertex_position[1].parse::<f32>()?;
                let z = vertex_position[2].parse::<f32>()?;
                verts.push(Vec3::new(x, y, z));
            } else if line.starts_with("f ") {
                let face_vertices = line.split_whitespace().skip(1).collect_vec();
                let mut face = vec![];
                for vertex in face_vertices {
                    let vertex_index = vertex.split('/').next().unwrap().parse::<usize>()? - 1;
                    face.push(vertex_index);
                }
                faces.push(face);
            }
        }
        Self::from_embedded_faces(&faces, &verts)
    }

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

    // Get normal of a given face, assumes the face is planar. If the face is not planar, then this function will not return the correct normal.
    // The normal is calculated as the cross product of two edges of the face; https://en.wikipedia.org/wiki/Normal_(geometry)
    #[must_use]
    pub fn normal(&self, id: FaceID) -> Vec3 {
        let vector_area = self.edges(id).iter().fold(Vec3::ZERO, |sum, &edge_id| {
            let u = self.vector(self.twin(edge_id));
            let v = self.vector(self.next(edge_id));
            sum + u.cross(v)
        });
        -vector_area.normalize()
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

    // Weight function
    pub fn weight_function_euclidean(&self) -> impl Fn(VertID, VertID) -> OrderedFloat<f32> + '_ {
        |a, b| OrderedFloat(self.position(a).distance(self.position(b)))
    }

    // Weight function
    pub fn weight_function_angle_edges(
        &self,
        slack: i32,
    ) -> impl Fn(EdgeID, EdgeID) -> OrderedFloat<f32> + '_ {
        move |a, b| OrderedFloat((self.vector(a).angle_between(self.vector(b))).powi(slack))
    }

    // Weight function
    pub fn weight_function_angle_edgepairs(
        &self,
        slack: i32,
    ) -> impl Fn((EdgeID, EdgeID), (EdgeID, EdgeID)) -> OrderedFloat<f32> + '_ {
        move |a, b| {
            let vector_a = self.midpoint(a.1) - self.midpoint(a.0);
            let vector_b = self.midpoint(b.1) - self.midpoint(b.0);
            OrderedFloat((vector_a.angle_between(vector_b)).powi(slack))
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

            let weight = vector_a.angle_between(vector_b).powi(angular_slack)
                + (vector_a.cross(self.edge_normal(a.0)).angle_between(axis)).powi(alignment_slack)
                + (vector_b.cross(self.edge_normal(b.0)).angle_between(axis)).powi(alignment_slack);

            OrderedFloat(weight)
        }
    }
}
