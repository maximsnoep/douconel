use bimap::BiHashMap;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::seq::IteratorRandom;
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
    pub struct EdgePairID;
}

pub type FaceMap = BiHashMap<usize, FaceID>;
pub type VertMap = BiHashMap<usize, VertID>;

// The doubly connected edge list (DCEL or Douconel), also known as half-edge data structure,
// is a data structure to represent an embedding of a planar graph in the plane, and polytopes in 3D.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Douconel<V, E, F> {
    pub verts: SlotMap<VertID, V>,
    pub edges: SlotMap<EdgeID, E>,
    pub faces: SlotMap<FaceID, F>,
    pub edgepairs: SlotMap<EdgePairID, (EdgeID, EdgeID)>,

    edge_root: SecondaryMap<EdgeID, VertID>,
    edge_face: SecondaryMap<EdgeID, FaceID>,
    edge_next: SecondaryMap<EdgeID, EdgeID>,
    edge_twin: SecondaryMap<EdgeID, EdgeID>,

    vert_rep: SecondaryMap<VertID, EdgeID>,
    face_rep: SecondaryMap<FaceID, EdgeID>,
}

impl<V, E, F> Douconel<V, E, F> {
    // Creates a new, empty Douconel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            verts: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            faces: SlotMap::with_key(),
            edgepairs: SlotMap::with_key(),
            edge_root: SecondaryMap::new(),
            edge_face: SecondaryMap::new(),
            edge_next: SecondaryMap::new(),
            edge_twin: SecondaryMap::new(),
            vert_rep: SecondaryMap::new(),
            face_rep: SecondaryMap::new(),
        }
    }

    // Verifies that all elements have their required properties set.
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

    /// Verifies that all references between elements are valid.
    pub fn verify_references(&self) -> Result<(), Box<dyn Error>> {
        for edge_id in self.edges.keys() {
            let _root_id = self.root(edge_id);
            let _face_id = self.face(edge_id);
            let _next_id = self.next(edge_id);
            let _twin_id = self.twin(edge_id);
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

    /// Verifies the invariants of the DCEL structure.
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

    // Returns the "representative" edge of the given vertex.
    // Panics if the vertex has no representative edge defined.
    #[must_use]
    pub fn vrep(&self, id: VertID) -> EdgeID {
        self.vert_rep.get(id).copied().unwrap_or_else(|| {
            panic!("V{id:?} has no representative edge defined.");
        })
    }

    // Returns the "representative" edge of the given face.
    // Panics if the face has no representative edge defined.
    #[must_use]
    pub fn frep(&self, id: FaceID) -> EdgeID {
        self.face_rep.get(id).copied().unwrap_or_else(|| {
            panic!("F:{id:?} has no representative edge defined.");
        })
    }

    // Returns the root vertex of the given edge.
    // Panics if the edge has no root defined or if the root does not exist.
    #[must_use]
    pub fn root(&self, id: EdgeID) -> VertID {
        let root_id = self
            .edge_root
            .get(id)
            .copied()
            .unwrap_or_else(|| panic!("E:{id:?} has no root defined."));

        assert!(
            self.verts.contains_key(root_id),
            "E:{id:?} has non-existing root"
        );

        root_id
    }

    // Returns the twin edge of the given edge.
    // Panics if the edge has no twin defined or if the twin does not exist.
    #[must_use]
    pub fn twin(&self, id: EdgeID) -> EdgeID {
        let twin_id = self.edge_twin.get(id).copied().unwrap_or_else(|| {
            panic!("E:{id:?} has no twin defined.");
        });

        assert!(
            self.edges.contains_key(twin_id),
            "E:{id:?} has non-existing twin"
        );

        twin_id
    }

    // Returns the next edge of the given edge.
    // Panics if the edge has no next defined or if the next does not exist.
    #[must_use]
    pub fn next(&self, id: EdgeID) -> EdgeID {
        let next_id = self.edge_next.get(id).copied().unwrap_or_else(|| {
            panic!("E:{id:?} has no next defined.");
        });

        assert!(
            self.edges.contains_key(next_id),
            "E:{id:?} has non-existing next"
        );

        next_id
    }

    // Returns the face of the given edge.
    // Panics if the edge has no face defined or if the face does not exist.
    #[must_use]
    pub fn face(&self, id: EdgeID) -> FaceID {
        let face_id = self.edge_face.get(id).copied().unwrap_or_else(|| {
            panic!("E:{id:?} has no face defined.");
        });

        assert!(
            self.faces.contains_key(face_id),
            "E:{id:?} has non-existing face"
        );

        face_id
    }

    // Returns the start and end vertex IDs of the given edge.
    // Panics if any of the roots are not defined or do not exist.
    #[must_use]
    pub fn endpoints(&self, id: EdgeID) -> (VertID, VertID) {
        (self.root(id), self.root(self.twin(id)))
    }

    // Returns the corner vertices of a given face.
    #[must_use]
    pub fn corners(&self, id: FaceID) -> Vec<VertID> {
        self.edges(id)
            .into_iter()
            .map(|edge_id| self.root(edge_id))
            .collect()
    }

    // Returns the outgoing edges of a given vertex.
    #[must_use]
    pub fn outgoing(&self, id: VertID) -> Vec<EdgeID> {
        let mut edges = vec![self.vrep(id)];
        loop {
            let twin = self.twin(edges.last().copied().unwrap_or_else(|| {
                panic!("{edges:?} should be non-empty");
            }));
            if edges.contains(&self.next(twin)) {
                return edges;
            }
            edges.push(self.next(twin));
        }
    }

    // Returns the edges of a given face.
    #[must_use]
    pub fn edges(&self, id: FaceID) -> Vec<EdgeID> {
        let mut edges = vec![self.frep(id)];
        loop {
            let next = self.next(edges.last().copied().unwrap_or_else(|| {
                panic!("{edges:?} should be non-empty");
            }));
            if next == self.frep(id) {
                return edges;
            }
            edges.push(next);
        }
    }

    // Returns the faces around a given vertex.
    #[must_use]
    pub fn star(&self, id: VertID) -> Vec<FaceID> {
        self.outgoing(id)
            .iter()
            .map(|&edge_id| self.face(edge_id))
            .collect()
    }

    // Returns the faces around a given vertex.
    #[must_use]
    pub fn faces(&self, id: EdgeID) -> [FaceID; 2] {
        [self.face(id), self.face(self.twin(id))]
    }

    // Returns the edge between the two vertices. Returns None if the vertices are not connected.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn vneighbors(&self, id: VertID) -> Vec<VertID> {
        let mut neighbors = Vec::new();
        for edge_id in self.outgoing(id) {
            neighbors.push(self.root(self.twin(edge_id)));
        }
        neighbors
    }

    // Returns the (edge-wise) neighbors of a given face.
    #[must_use]
    pub fn fneighbors(&self, id: FaceID) -> Vec<FaceID> {
        let mut neighbors = Vec::new();
        for edge_id in self.edges(id) {
            neighbors.push(self.face(self.twin(edge_id)));
        }
        neighbors
    }

    // Returns the number of vertices in the mesh.
    #[must_use]
    pub fn nr_verts(&self) -> usize {
        self.verts.len()
    }

    // Returns the number of (half)edges in the mesh.
    #[must_use]
    pub fn nr_edges(&self) -> usize {
        self.edges.len()
    }

    // Returns the number of faces in the mesh.
    #[must_use]
    pub fn nr_faces(&self) -> usize {
        self.faces.len()
    }

    // Return `n` random vertices.
    #[must_use]
    pub fn random_verts(&self, n: usize) -> Vec<VertID> {
        let mut rng = rand::thread_rng();
        self.verts.keys().choose_multiple(&mut rng, n)
    }

    // Return `n` random edges.
    #[must_use]
    pub fn random_edges(&self, n: usize) -> Vec<EdgeID> {
        let mut rng = rand::thread_rng();
        self.edges.keys().choose_multiple(&mut rng, n)
    }

    // Return `n` random faces.
    #[must_use]
    pub fn random_faces(&self, n: usize) -> Vec<FaceID> {
        let mut rng = rand::thread_rng();
        self.faces.keys().choose_multiple(&mut rng, n)
    }

    pub fn neighbor_function_primal(&self) -> impl Fn(VertID) -> Vec<VertID> + '_ {
        |v_id| self.vneighbors(v_id)
    }

    pub fn neighbor_function_edgegraph(&self) -> impl Fn(EdgeID) -> Vec<EdgeID> + '_ {
        |e_id| self.outgoing(self.endpoints(e_id).1)
    }

    pub fn neighbor_function_edgepairgraph(
        &self,
    ) -> impl Fn((EdgeID, EdgeID)) -> Vec<(EdgeID, EdgeID)> + '_ {
        |(_, to)| {
            let next = self.twin(to);
            self.edges(self.face(next))
                .into_iter()
                .filter(|&edge_id| edge_id != next)
                .map(|next_to| (next, next_to))
                .collect()
        }
    }
}

// Construct a DCEL from a list of faces, where each face is a list of vertex indices.
impl<V: Default, E: Default, F: Default> Douconel<V, E, F> {
    pub fn from_faces(faces: &[Vec<usize>]) -> Result<(Self, VertMap, FaceMap), Box<dyn Error>> {
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
        let mut vertex_pointers = VertMap::new();
        let mut face_pointers = FaceMap::new();

        let vertices = faces.iter().flatten().unique().copied().collect_vec();

        for inp_vert_id in vertices {
            let vert_id = mesh.verts.insert(V::default());
            vertex_pointers.insert(inp_vert_id, vert_id);
        }

        // 2. Create the faces with its (half)edges.
        // Need mapping between vertices and edges
        let mut vert_to_edge = HashMap::<VertID, EdgeID>::new();
        let mut face_to_edge = HashMap::<FaceID, EdgeID>::new();
        let mut endpoints_to_edges = HashMap::<(VertID, VertID), EdgeID>::new();
        for (inp_face_id, inp_face_edges) in faces.iter().enumerate() {
            let face_id = mesh.faces.insert(F::default());

            println!("face: {inp_face_id:?} edges: {inp_face_edges:?}");

            face_pointers.insert(inp_face_id, face_id);

            let mut conc = inp_face_edges.clone();
            conc.push(inp_face_edges[0]); // Re-append the first to loop back

            let edges = conc
                .iter()
                .tuple_windows()
                .map(|(inp_start_vertex, inp_end_vertex)| {
                    (
                        vertex_pointers
                            .get_by_left(inp_start_vertex)
                            .copied()
                            .unwrap_or_else(|| {
                                panic!("V:{inp_start_vertex:?} does not have a vertex pointer")
                            }),
                        vertex_pointers
                            .get_by_left(inp_end_vertex)
                            .copied()
                            .unwrap_or_else(|| {
                                panic!("V:{inp_end_vertex:?} does not have a vertex pointer")
                            }),
                    )
                })
                .collect_vec();

            let mut edge_ids = Vec::with_capacity(edges.len());
            for (start_vertex, end_vertex) in edges {
                let edge_id = mesh.edges.insert(E::default());

                if endpoints_to_edges
                    .insert((start_vertex, end_vertex), edge_id)
                    .is_some()
                {
                    bail!("Edge for ({start_vertex:?}, {end_vertex:?}) already exists");
                };

                mesh.edge_root.insert(edge_id, start_vertex);
                mesh.edge_face.insert(edge_id, face_id);
                vert_to_edge.insert(start_vertex, edge_id);
                edge_ids.push(edge_id);
            }
            face_to_edge.insert(face_id, *edge_ids.first().unwrap());

            // Linking each edge to its next edge in the face
            for edge_index in 0..edge_ids.len() {
                mesh.edge_next.insert(
                    edge_ids[edge_index],
                    edge_ids[(edge_index + 1) % edge_ids.len()],
                );
            }
        }

        // 3. Assign representatives to vertices and faces
        for vert_id in mesh.verts.keys() {
            mesh.vert_rep.insert(
                vert_id,
                vert_to_edge.get(&vert_id).copied().unwrap_or_else(|| {
                    panic!("V:{vert_id:?} has no representative edge");
                }),
            );
        }
        for face_id in mesh.faces.keys() {
            mesh.face_rep.insert(
                face_id,
                face_to_edge.get(&face_id).copied().unwrap_or_else(|| {
                    panic!("F:{face_id:?} has no representative edge");
                }),
            );
        }

        // 4. Assign twins.
        for (&(vert_a, vert_b), &edge_id) in &endpoints_to_edges {
            // Retrieve the twin edge
            let twin_id = endpoints_to_edges
                .get(&(vert_b, vert_a))
                .copied()
                .unwrap_or_else(|| {
                    panic!("Edge for (V:{vert_a:?}, V:{vert_b:?}) does not have a twin")
                });

            // Assign twins
            mesh.edge_twin.insert(edge_id, twin_id);
            mesh.edge_twin.insert(twin_id, edge_id);
        }

        Ok((mesh, vertex_pointers, face_pointers))
    }
}

// Find the shortest path from element `a` to `b` using Dijkstra's algorithm.
// Neighborhood of a vertex is defined by `neighbor_function`, and the weight a pair elements is defined by `weight_function
pub fn find_shortest_path<T: std::cmp::Eq + std::hash::Hash + std::clone::Clone + Copy>(
    a: T,
    b: T,
    neighbor_function: impl Fn(T) -> Vec<T>,
    weight_function: impl Fn(T, T) -> OrderedFloat<f32>,
    cache: &mut HashMap<T, Vec<(T, OrderedFloat<f32>)>>,
) -> Option<(Vec<T>, OrderedFloat<f32>)> {
    pathfinding::prelude::dijkstra(
        &a,
        |&elem| {
            if cache.contains_key(&elem) {
                cache[&elem].clone()
            } else {
                let neighbors = neighbor_function(elem)
                    .iter()
                    .map(|&neighbor| (neighbor, weight_function(elem, neighbor)))
                    .collect_vec();
                cache.insert(elem, neighbors.clone());
                neighbors
            }
        },
        |&elem| elem == b,
    )
}

// Find the shortest cycle through element `a`, using the `find_shortest_path` function.
pub fn find_shortest_cycle<T: std::cmp::Eq + std::hash::Hash + std::clone::Clone + Copy>(
    a: T,
    neighbor_function: impl Fn(T) -> Vec<T>,
    weight_function: impl Fn(T, T) -> OrderedFloat<f32>,
    cache: &mut HashMap<T, Vec<(T, OrderedFloat<f32>)>>,
) -> Option<(Vec<T>, OrderedFloat<f32>)> {
    neighbor_function(a)
        .iter()
        .filter_map(|&neighbor| {
            find_shortest_path(neighbor, a, &neighbor_function, &weight_function, cache)
        })
        .sorted_by(|(_, cost1), (_, cost2)| cost1.cmp(cost2))
        .next()
        .map(|(path, score)| ([vec![a], path].concat(), score))
}
