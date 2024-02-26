use crate::douconel::{Douconel, EdgeID, FaceID, VertID};
use glam::Vec3;

/// --- Vertices with defined position ---
pub trait HasPosition {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

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
    pub fn position(&self, id: VertID) -> Vec3 {
        self.verts[id].position()
    }

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
    // See https://en.wikipedia.org/wiki/Angular_defect
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
}

/// --- Faces with defined normal ---
pub trait HasNormal {
    fn normal(&self) -> Vec3;
    fn set_normal(&mut self, normal: Vec3);
}

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
