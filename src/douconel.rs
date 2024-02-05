use itertools::Itertools;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use slotmap::SecondaryMap;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

slotmap::new_key_type! {
    pub struct VertID;
    pub struct EdgeID;
    pub struct FaceID;
}

// The doubly connected edge list (DCEL or Douconel), also known as half-edge data structure,
// is a data structure to represent an embedding of a planar graph in the plane, and polytopes in 3D.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Douconel<V, E, F> {
    pub verts: SlotMap<VertID, V>,
    pub edges: SlotMap<EdgeID, E>,
    pub faces: SlotMap<FaceID, F>,

    edge_root: SecondaryMap<EdgeID, VertID>,
    edge_face: SecondaryMap<EdgeID, FaceID>,
    edge_next: SecondaryMap<EdgeID, EdgeID>,
    edge_twin: SecondaryMap<EdgeID, EdgeID>,

    vert_rep: SecondaryMap<VertID, EdgeID>,
    face_rep: SecondaryMap<FaceID, EdgeID>,
}

impl<V, E, F> Douconel<V, E, F> {
    pub fn new() -> Self {
        Self {
            verts: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            faces: SlotMap::with_key(),
            edge_root: SecondaryMap::new(),
            edge_face: SecondaryMap::new(),
            edge_next: SecondaryMap::new(),
            edge_twin: SecondaryMap::new(),
            vert_rep: SecondaryMap::new(),
            face_rep: SecondaryMap::new(),
        }
    }

    // iterate through all elements, and assure that all elements have set properties
    pub fn verify_properties(&self) -> Result<(), Box<dyn Error>> {
        for edge_id in self.edges.keys() {
            if !self.edge_root.contains_key(edge_id) {
                bail!("Edge {edge_id:?} has no root");
            }
            if !self.edge_face.contains_key(edge_id) {
                bail!("Edge {edge_id:?} has no face");
            }
            if !self.edge_next.contains_key(edge_id) {
                bail!("Edge {edge_id:?} has no next");
            }
            if !self.edge_twin.contains_key(edge_id) {
                bail!("Edge {edge_id:?} has no twin");
            }
        }
        for vert_id in self.verts.keys() {
            if !self.vert_rep.contains_key(vert_id) {
                bail!("Vert {vert_id:?} has no rep");
            }
        }
        for face_id in self.faces.keys() {
            if !self.face_rep.contains_key(face_id) {
                bail!("Face {face_id:?} has no rep");
            }
        }

        Ok(())
    }

    // iterate through all elements, and assure that all references are existing
    pub fn verify_references(&self) -> Result<(), Box<dyn Error>> {
        for edge_id in self.edges.keys() {
            let root_id = self.edge_root[edge_id];
            let face_id = self.edge_face[edge_id];
            let next_id = self.edge_next[edge_id];
            let twin_id = self.edge_twin[edge_id];

            if !self.verts.contains_key(root_id) {
                bail!("Edge {edge_id:?} has non-existing root");
            }
            if !self.faces.contains_key(face_id) {
                bail!("Edge {edge_id:?} has non-existing face");
            }
            if !self.edges.contains_key(next_id) {
                bail!("Edge {edge_id:?} has non-existing next");
            }
            if !self.edges.contains_key(twin_id) {
                bail!("Edge {edge_id:?} has non-existing twin");
            }
        }
        for vert_id in self.verts.keys() {
            let rep_id = self.vert_rep[vert_id];
            if !self.edges.contains_key(rep_id) {
                bail!("Vert {vert_id:?} has non-existing rep");
            }
        }
        for face_id in self.faces.keys() {
            let rep_id = self.face_rep[face_id];
            if !self.edges.contains_key(rep_id) {
                bail!("Face {face_id:?} has non-existing rep");
            }
        }

        Ok(())
    }

    pub fn verify_invariants(&self) -> Result<(), Box<dyn Error>> {
        // this->twin->twin == this
        for edge_id in self.edges.keys() {
            let twin_id = self.twin(edge_id);
            let twin_twin_id = self.twin(twin_id);
            if twin_twin_id != edge_id {
                bail!("Edge {edge_id:?}: [this->twin->twin == this] violated");
            }
        }

        // this->twin->next->root == this->root
        for edge_id in self.edges.keys() {
            let root_id = self.root(edge_id);
            let twin_id = self.twin(edge_id);
            let twin_next_id = self.next(twin_id);
            let twin_next_root_id = self.root(twin_next_id);
            if twin_next_root_id != root_id {
                bail!("Edge {edge_id:?}: [this->twin->next->root == this->root] violated");
            }
        }

        // this->next->face == this->face
        for edge_id in self.edges.keys() {
            let face_id = self.face(edge_id);
            let next_id = self.next(edge_id);
            let next_face_id = self.face(next_id);
            if next_face_id != face_id {
                bail!("Edge {edge_id:?}: [this->next->face == this->face] violated");
            }
        }

        // this->next->...->next == this
        let max_face_size = 100;
        'outer: for edge_id in self.edges.keys() {
            let mut next_id = edge_id;
            for _ in 0..max_face_size {
                next_id = self.next(next_id);
                if next_id == edge_id {
                    continue 'outer;
                }
            }
            bail!("Edge {edge_id:?}: [this->next->...->next == this] violated");
        }

        Ok(())
    }

    // Returns the "representative" edge
    pub fn vrep(&self, id: VertID) -> EdgeID {
        self.vert_rep[id]
    }
    pub fn frep(&self, id: FaceID) -> EdgeID {
        self.face_rep[id]
    }

    // Returns the root vertex of the given edge.
    pub fn root(&self, id: EdgeID) -> VertID {
        self.edge_root[id]
    }

    // Returns the twin edge of the given edge.
    pub fn twin(&self, id: EdgeID) -> EdgeID {
        self.edge_twin[id]
    }

    // Returns the next edge of the given edge.
    pub fn next(&self, id: EdgeID) -> EdgeID {
        self.edge_next[id]
    }

    // Returns the face of the given edge.
    pub fn face(&self, id: EdgeID) -> FaceID {
        self.edge_face[id]
    }

    // Returns the start and end vertex IDs of the given edge.
    pub fn endpoints(&self, id: EdgeID) -> (VertID, VertID) {
        (self.root(id), self.root(self.twin(id)))
    }

    // Returns the corner vertices of a given face.
    pub fn corners(&self, id: FaceID) -> Vec<VertID> {
        let mut vertices = Vec::new();
        for edge_id in self.edges(id.into()) {
            vertices.push(self.root(edge_id));
        }
        vertices
    }

    // Returns the outgoing edges of a given vertex.
    pub fn outgoing(&self, id: VertID) -> Vec<EdgeID> {
        let mut edges = vec![self.vrep(id)];
        loop {
            let twin = self.twin(edges.last().copied().unwrap());
            let next = self.next(twin);
            if edges.contains(&next) {
                return edges;
            }
            edges.push(next);
        }
    }

    // Returns the edges of a given face.
    pub fn edges(&self, id: FaceID) -> Vec<EdgeID> {
        let mut edges = vec![self.frep(id)];
        loop {
            let next = self.next(edges.last().copied().unwrap());
            if edges.contains(&next) {
                return edges;
            }
            edges.push(next);
        }
    }

    // Returns the faces around a given vertex.
    pub fn star(&self, id: VertID) -> Vec<FaceID> {
        let mut faces = Vec::new();
        for edge_id in self.outgoing(id) {
            faces.push(self.face(edge_id));
        }
        faces
    }

    // Returns the faces around a given vertex.
    pub fn faces(&self, id: EdgeID) -> (FaceID, FaceID) {
        (self.face(id), self.face(self.twin(id)))
    }

    // Returns the edge between the two vertices. Returns None if the vertices are not connected.
    pub fn edge_between_verts(&self, id_a: VertID, id_b: VertID) -> Option<(EdgeID, EdgeID)> {
        let edges_a = self.outgoing(id_a);
        let edges_b = self.outgoing(id_b);
        for &edge_a_id in &edges_a {
            for &edge_b_id in &edges_b {
                if self.twin(edge_a_id) == edge_b_id {
                    return Some((edge_a_id, edge_b_id));
                }
            }
        }
        None
    }

    // Returns the edge between the two faces. Returns None if the faces do not share an edge.
    pub fn edge_between_faces(&self, id_a: FaceID, id_b: FaceID) -> Option<(EdgeID, EdgeID)> {
        let edges_a = self.edges(id_a);
        let edges_b = self.edges(id_b);
        for &edge_a_id in &edges_a {
            for &edge_b_id in &edges_b {
                if self.twin(edge_a_id) == edge_b_id {
                    return Some((edge_a_id, edge_b_id));
                }
            }
        }
        None
    }

    // Returns the neighbors of a given vertex.
    pub fn vneighbors(&self, id: VertID) -> Vec<VertID> {
        let mut neighbors = Vec::new();
        for edge_id in self.outgoing(id) {
            neighbors.push(self.root(self.twin(edge_id)));
        }
        neighbors
    }

    // Returns the neighbors of a given face.
    pub fn fneighbors(&self, id: FaceID) -> Vec<FaceID> {
        let mut neighbors = Vec::new();
        for edge_id in self.edges(id.into()) {
            neighbors.push(self.face(self.twin(edge_id)));
        }
        neighbors
    }

    // Returns the number of vertices in the mesh.
    pub fn nr_verts(&self) -> usize {
        self.verts.len()
    }

    // Returns the number of (half)edges in the mesh.
    pub fn nr_edges(&self) -> usize {
        self.edges.len()
    }

    // Returns the number of faces in the mesh.
    pub fn nr_faces(&self) -> usize {
        self.faces.len()
    }
}

impl<V: Default, E: Default, F: Default> Douconel<V, E, F> {
    pub fn from_faces(
        faces: Vec<Vec<usize>>,
    ) -> Result<(Self, HashMap<usize, VertID>, HashMap<usize, FaceID>), Box<dyn Error>> {
        let mut mesh = Self::new();

        // 1. Create the vertices.
        //      trivial; get all unique input vertices (from the faces), and create a vertex for each of them
        //
        // 2. Create the faces with its (half)edges.
        //      each face has edges defined by a sequence of vertices, example:
        //          face = [v0, v1, v2]
        //          then we create three edges = [(v0, v1), (v1, v2), (v2, v0)]
        //                v0
        //                *
        //               ^ \
        //              /   \ e0
        //          e2 /     \
        //            /       v
        //        v2 * < - - - * v1
        //                e1
        //
        // 3. Assign representatives to vertices.
        //      trivial; just assign some edge that has this vertex as its root (just requires some bookkeeping)
        //      return error if no such edge exists
        //
        // 4. Assign twins.
        //      trivial; just assign THE edge that has the same endpoints, but swapped (just requires some bookkeeping)
        //      return error if no such edge exists
        //

        // 1. Create the vertices.
        // Need mapping between original indices, and new pointers
        let mut vertex_pointers = HashMap::<usize, VertID>::new();
        let mut face_pointers = HashMap::<usize, FaceID>::new();

        let vertices = faces
            .iter()
            .flatten()
            .unique()
            .map(|&inp_vert_id| inp_vert_id)
            .collect_vec();

        for inp_vert_id in vertices {
            let vert_id = mesh.verts.insert(V::default());
            vertex_pointers.insert(inp_vert_id, vert_id);
        }

        // 2. Create the faces with its (half)edges.
        // Need mapping between vertices and edges
        let mut root_to_edges = HashMap::<VertID, Vec<EdgeID>>::new();
        let mut endpoints_to_edges = HashMap::<(VertID, VertID), Vec<EdgeID>>::new();
        for (inp_face_id, inp_face_edges) in faces.iter().enumerate() {
            let face_id = mesh.faces.insert(F::default());
            face_pointers.insert(inp_face_id, face_id);

            let mut conc = inp_face_edges.clone();
            conc.push(inp_face_edges[0]); // Re-append the first to loop back

            let edges = conc
                .iter()
                .tuple_windows()
                .map(|(inp_start_vertex, inp_end_vertex)| {
                    (
                        vertex_pointers[inp_start_vertex],
                        vertex_pointers[inp_end_vertex],
                    )
                })
                .collect_vec();

            let mut edge_ids = Vec::with_capacity(edges.len());

            for (start_vertex, end_vertex) in edges {
                let edge_id = mesh.edges.insert(E::default());

                root_to_edges
                    .entry(start_vertex)
                    .or_insert(Vec::new())
                    .push(edge_id);

                endpoints_to_edges
                    .entry((start_vertex, end_vertex))
                    .or_insert(Vec::new())
                    .push(edge_id);

                mesh.edge_root.insert(edge_id, start_vertex);
                mesh.edge_face.insert(edge_id, face_id);

                edge_ids.push(edge_id);
            }

            // Set the representative edge for the face
            mesh.face_rep.insert(face_id, edge_ids[0]);

            // Linking each edge to its next edge in the face
            edge_ids.push(edge_ids[0]); // Re-append the first to loop back
            for (edge_a, edge_b) in edge_ids.into_iter().tuple_windows() {
                mesh.edge_next.insert(edge_a, edge_b);
            }
        }

        // 3. Assign representatives to vertices.
        for (vert_id, edge_ids) in root_to_edges {
            mesh.vert_rep.insert(vert_id, edge_ids[0]);
        }

        // 4. Assign twins.
        for (&(vert_a, vert_b), edge_ids) in &endpoints_to_edges {
            // Check for the expected single edge id
            if edge_ids.len() != 1 {
                bail!(
                    "Expected 1 edge_id for ({:?}, {:?}), found {}",
                    vert_a,
                    vert_b,
                    edge_ids.len()
                );
            }
            let edge_id = edge_ids[0];

            // Retrieve the twin edge
            let twin_id = match endpoints_to_edges.get(&(vert_b, vert_a)) {
                Some(ids) => {
                    if ids.len() != 1 {
                        bail!(
                            "Expected 1 twin_id for ({:?}, {:?}), found {}",
                            vert_b,
                            vert_a,
                            ids.len()
                        );
                    }
                    ids[0]
                }
                None => bail!("Twin edge for ({:?}, {:?}) not found", vert_a, vert_b),
            };

            // Assign twins
            mesh.edge_twin.insert(edge_id, twin_id);
            mesh.edge_twin.insert(twin_id, edge_id);
        }

        println!(
            "Constructed valid douconel: |V|={}, |hE|={}, |F|={}, ",
            mesh.verts.len(),
            mesh.edges.len(),
            mesh.faces.len()
        );

        Ok((mesh, vertex_pointers, face_pointers))
    }
}
