use itertools::Itertools;
use simple_error::bail;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

slotmap::new_key_type! {
    pub struct VertID;
    pub struct EdgeID;
    pub struct FaceID;
}

pub enum ElemID {
    Vert(VertID),
    Edge(EdgeID),
    Face(FaceID),
}

impl From<VertID> for ElemID {
    fn from(id: VertID) -> Self {
        ElemID::Vert(id)
    }
}

impl From<EdgeID> for ElemID {
    fn from(id: EdgeID) -> Self {
        ElemID::Edge(id)
    }
}

impl From<FaceID> for ElemID {
    fn from(id: FaceID) -> Self {
        ElemID::Face(id)
    }
}

#[derive(Default, Copy, Clone)]
pub struct Edge<T> {
    root: Option<VertID>,
    face: Option<FaceID>,
    next: Option<EdgeID>,
    twin: Option<EdgeID>,

    pub aux: T,
}

#[derive(Default, Copy, Clone)]
pub struct Vert<T> {
    rep: Option<EdgeID>,

    pub aux: T,
}

#[derive(Default, Copy, Clone)]
pub struct Face<T> {
    rep: Option<EdgeID>,

    pub aux: T,
}

// The doubly connected edge list (DCEL or Douconel), also known as half-edge data structure,
// is a data structure to represent an embedding of a planar graph in the plane, and polytopes in 3D.
#[derive(Default, Clone)]
pub struct Douconel<V, E, F> {
    pub verts: SlotMap<VertID, Vert<V>>,
    pub edges: SlotMap<EdgeID, Edge<E>>,
    pub faces: SlotMap<FaceID, Face<F>>,
}

impl<V: Default + Copy + Clone, E: Default + Copy + Clone, F: Default + Copy + Clone>
    Douconel<V, E, F>
{
    pub fn new() -> Self {
        Self {
            verts: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            faces: SlotMap::with_key(),
        }
    }

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
            let vert_id = mesh.verts.insert(Vert::default());
            vertex_pointers.insert(inp_vert_id, vert_id);
        }

        // 2. Create the faces with its (half)edges.
        // Need mapping between vertices and edges
        let mut root_to_edges = HashMap::<VertID, Vec<EdgeID>>::new();
        let mut endpoints_to_edges = HashMap::<(VertID, VertID), Vec<EdgeID>>::new();
        for (inp_face_id, inp_face_edges) in faces.iter().enumerate() {
            let face_id = mesh.faces.insert(Face::default());
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
                let edge_id = mesh.edges.insert(Edge::default());

                root_to_edges
                    .entry(start_vertex)
                    .or_insert(Vec::new())
                    .push(edge_id);

                endpoints_to_edges
                    .entry((start_vertex, end_vertex))
                    .or_insert(Vec::new())
                    .push(edge_id);

                if let Some(edge) = mesh.edges.get_mut(edge_id) {
                    edge.root = Some(start_vertex);
                    edge.face = Some(face_id);
                } else {
                    panic!("Edge {edge_id:?} does not exist in the mesh");
                }

                edge_ids.push(edge_id);
            }

            // Linking each edge to its next edge in the face
            edge_ids.push(edge_ids[0]); // Re-append the first to loop back
            for window in edge_ids.windows(2) {
                if let [edge_a, edge_b] = *window {
                    if let Some(edge) = mesh.edges.get_mut(edge_a) {
                        edge.next = Some(edge_b);
                    } else {
                        bail!("Edge {edge_a:?} does not exist in the mesh");
                    }
                }
            }

            // Set the representative edge for the face
            if let Some(face) = mesh.faces.get_mut(face_id) {
                face.rep = edge_ids.first().copied();
            } else {
                bail!("Face {face_id:?} does not exist in the mesh");
            }
        }

        // 3. Assign representatives to vertices.
        for (vert_id, edges) in root_to_edges {
            if let Some(vert) = mesh.verts.get_mut(vert_id) {
                vert.rep = Some(edges[0]);
            } else {
                bail!("Vertex {vert_id:?} does not exist in the mesh");
            }
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
            if let Some(edge) = mesh.edges.get_mut(edge_id) {
                edge.twin = Some(twin_id);
            } else {
                bail!("Edge {edge_id:?} does not exist in the mesh");
            }

            if let Some(twin) = mesh.edges.get_mut(twin_id) {
                twin.twin = Some(edge_id);
            } else {
                bail!("Twin edge {twin_id:?} does not exist in the mesh");
            }
        }

        println!(
            "Constructed valid douconel: |V|={}, |hE|={}, |F|={}, ",
            mesh.verts.len(),
            mesh.edges.len(),
            mesh.faces.len()
        );

        Ok((mesh, vertex_pointers, face_pointers))
    }

    // iterate through all elements, and assure that no None values exist
    pub fn verify_correctness(&self) -> Result<(), Box<dyn Error>> {
        for (edge_id, edge) in &self.edges {
            if edge.root.is_none() {
                bail!("Edge {edge_id:?} has no root");
            }
            if edge.face.is_none() {
                bail!("Edge {edge_id:?} has no face");
            }
            if edge.next.is_none() {
                bail!("Edge {edge_id:?} has no next");
            }
            if edge.twin.is_none() {
                bail!("Edge {edge_id:?} has no twin");
            }
        }
        for (vert_id, vert) in &self.verts {
            if vert.rep.is_none() {
                bail!("Vert {vert_id:?} has no rep");
            }
        }
        for (face_id, face) in &self.faces {
            if face.rep.is_none() {
                bail!("Face {face_id:?} has no rep");
            }
        }

        Ok(())
    }

    pub fn verify_invariants(&self) -> Result<(), Box<dyn Error>> {
        // this->twin->twin == this
        for edge_id in self.edges.keys() {
            if let Some(twin_id) = self.twin(edge_id) {
                if let Some(twin_twin_id) = self.twin(twin_id) {
                    if twin_twin_id != edge_id {
                        bail!("Edge {edge_id:?}: [this->twin->twin == this] violated");
                    }
                }
            }
        }

        // this->twin->next->root == this->root
        for edge_id in self.edges.keys() {
            if let Some(root_id) = self.root(edge_id) {
                if let Some(twin_id) = self.twin(edge_id) {
                    if let Some(twin_next_id) = self.next(twin_id) {
                        if let Some(twin_next_root_id) = self.root(twin_next_id) {
                            if twin_next_root_id != root_id {
                                bail!("Edge {edge_id:?}: [this->twin->next->root == this->root] violated");
                            }
                        }
                    }
                }
            }
        }

        // this->next->face == this->face
        for edge_id in self.edges.keys() {
            if let Some(face_id) = self.face(edge_id) {
                if let Some(next_id) = self.next(edge_id) {
                    if let Some(next_face_id) = self.face(next_id) {
                        if next_face_id != face_id {
                            bail!("Edge {edge_id:?}: [this->next->face == this->face] violated");
                        }
                    }
                }
            }
        }

        // this->next->...->next == this
        let max_face_size = 100;
        'outer: for edge_id in self.edges.keys() {
            let mut next_id = edge_id;
            for _ in 0..max_face_size {
                if let Some(next_next_id) = self.next(next_id) {
                    if next_next_id == edge_id {
                        continue 'outer;
                    }
                    next_id = next_next_id;
                }
            }
            bail!("Edge {edge_id:?}: [this->next->...->next == this] violated");
        }

        Ok(())
    }

    // Returns the "representative" edge
    pub fn rep(&self, id: ElemID) -> Option<EdgeID> {
        match id {
            ElemID::Vert(id) => self.verts.get(id)?.rep,
            ElemID::Face(id) => self.faces.get(id)?.rep,
            _ => None,
        }
    }

    // Returns the root vertex of the given edge.
    pub fn root(&self, id: EdgeID) -> Option<VertID> {
        self.edges.get(id)?.root
    }

    // Returns the twin edge of the given edge.
    pub fn twin(&self, id: EdgeID) -> Option<EdgeID> {
        self.edges.get(id)?.twin
    }

    // Returns the next edge of the given edge.
    pub fn next(&self, id: EdgeID) -> Option<EdgeID> {
        self.edges.get(id)?.next
    }

    // Returns the face of the given edge.
    pub fn face(&self, id: EdgeID) -> Option<FaceID> {
        self.edges.get(id)?.face
    }

    // Returns the vertices of a given element.
    pub fn vertices(&self, id: ElemID) -> Option<Vec<VertID>> {
        match id {
            ElemID::Edge(id) => Some(self.endpoints(id)?),
            ElemID::Face(id) => Some(self.vertices_of_face(id)?),
            _ => None,
        }
    }

    // Returns the start and end vertex IDs of the given edge.
    pub fn endpoints(&self, id: EdgeID) -> Option<Vec<VertID>> {
        Some(vec![self.root(id)?, self.root(self.twin(id)?)?])
    }

    // Returns the vertices of a given face.
    pub fn vertices_of_face(&self, id: FaceID) -> Option<Vec<VertID>> {
        let mut vertices = Vec::new();
        for edge_id in self.edges(id.into())? {
            vertices.push(self.root(edge_id)?);
        }
        Some(vertices)
    }

    // Returns the edges around a given element.
    pub fn edges(&self, id: ElemID) -> Option<Vec<EdgeID>> {
        match id {
            ElemID::Vert(id) => self.edges_of_vert(id),
            ElemID::Face(id) => self.edges_of_face(id),
            _ => None,
        }
    }

    // Returns the edges around a given vertex.
    pub fn edges_of_vert(&self, id: VertID) -> Option<Vec<EdgeID>> {
        let mut edges = vec![self.rep(ElemID::Vert(id))?];
        loop {
            let twin = self.twin(*edges.last().unwrap())?;
            let next = self.next(twin)?;
            if edges.contains(&next) {
                return Some(edges);
            }
            edges.push(next);
        }
    }

    // Returns the edges around a given face.
    pub fn edges_of_face(&self, id: FaceID) -> Option<Vec<EdgeID>> {
        let mut edges = vec![self.rep(ElemID::Face(id))?];
        loop {
            let next = self.next(*edges.last().unwrap())?;
            if edges.contains(&next) {
                return Some(edges);
            }
            edges.push(next);
        }
    }

    // Returns the faces around a given element.
    pub fn faces(&self, id: ElemID) -> Option<Vec<FaceID>> {
        match id {
            ElemID::Vert(id) => Some(self.faces_of_vert(id)?),
            ElemID::Edge(id) => Some(self.faces_of_edge(id)?),
            _ => None,
        }
    }

    // Returns the faces around a given vert.
    pub fn faces_of_vert(&self, id: VertID) -> Option<Vec<FaceID>> {
        let mut faces = Vec::new();
        for edge_id in self.edges(id.into())? {
            if let Some(face_id) = self.face(edge_id) {
                faces.push(face_id);
            }
        }
        Some(faces)
    }

    // Returns the faces around a given edge.
    pub fn faces_of_edge(&self, id: EdgeID) -> Option<Vec<FaceID>> {
        Some(vec![self.face(id)?, self.face(self.twin(id)?)?])
    }

    // Returns the edge between the two elements.
    pub fn between(&self, id_a: ElemID, id_b: ElemID) -> Option<[EdgeID; 2]> {
        match (id_a, id_b) {
            (ElemID::Vert(id_a), ElemID::Vert(id_b)) => self.edge_between_verts(id_a, id_b),
            (ElemID::Face(id_a), ElemID::Face(id_b)) => self.edge_between_faces(id_a, id_b),
            _ => None,
        }
    }

    // Returns the edge between the two vertices.
    pub fn edge_between_verts(&self, id_a: VertID, id_b: VertID) -> Option<[EdgeID; 2]> {
        let edges_a = self.edges_of_vert(id_a)?;
        let edges_b = self.edges_of_vert(id_b)?;
        for &edge_a_id in &edges_a {
            for &edge_b_id in &edges_b {
                if self.twin(edge_a_id)? == edge_b_id {
                    return Some([edge_a_id, edge_b_id]);
                }
            }
        }
        None
    }

    // Returns the edge between the two faces.
    pub fn edge_between_faces(&self, id_a: FaceID, id_b: FaceID) -> Option<[EdgeID; 2]> {
        let edges_a = self.edges_of_face(id_a)?;
        let edges_b = self.edges_of_face(id_b)?;
        for &edge_a_id in &edges_a {
            for &edge_b_id in &edges_b {
                if self.twin(edge_a_id)? == edge_b_id {
                    return Some([edge_a_id, edge_b_id]);
                }
            }
        }
        None
    }

    // Returns the neighbors of a given element.
    pub fn neighbors(&self, id: ElemID) -> Option<Vec<ElemID>> {
        match id {
            ElemID::Vert(id) => Some(
                self.neighbors_of_vert(id)?
                    .into_iter()
                    .map(|id| ElemID::Vert(id))
                    .collect(),
            ),
            ElemID::Face(id) => Some(
                self.neighbors_of_face(id)?
                    .into_iter()
                    .map(|id| ElemID::Face(id))
                    .collect(),
            ),
            _ => None,
        }
    }

    // Returns the neighbors of a given vertex.
    pub fn neighbors_of_vert(&self, id: VertID) -> Option<Vec<VertID>> {
        let mut neighbors = Vec::new();
        for edge_id in self.edges(id.into())? {
            neighbors.push(self.root(self.twin(edge_id)?)?);
        }
        Some(neighbors)
    }

    // Returns the neighbors of a given face.
    pub fn neighbors_of_face(&self, id: FaceID) -> Option<Vec<FaceID>> {
        let mut neighbors = Vec::new();
        for edge_id in self.edges(id.into())? {
            neighbors.push(self.face(self.twin(edge_id)?)?);
        }
        Some(neighbors)
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
