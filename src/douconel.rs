use bevy::prelude::*;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::mem::Memory;

#[derive(Default, Clone, Debug)]
struct EdgePtr(Option<usize>);

#[derive(Default, Clone, Debug, Eq, Hash, PartialEq)]
struct VrtxPtr(Option<usize>);

#[derive(Default, Clone, Debug)]
struct FacePtr(Option<usize>);

#[derive(Default, Clone, Debug)]
pub struct Edge<T> {
    pub root: VrtxPtr,
    pub face: FacePtr,
    pub next: EdgePtr,
    pub twin: EdgePtr,

    pub aux: T,
}

impl<T: std::default::Default> Edge<T> {
    pub fn new(root: VrtxPtr) -> Self {
        Self {
            root,
            face: FacePtr(None),
            next: EdgePtr(None),
            twin: EdgePtr(None),
            aux: Default::default(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Vrtx<T> {
    pub rep: EdgePtr,
    pub pos: Vec3,

    pub aux: T,
}

impl<T: std::default::Default> Vrtx<T> {
    pub fn new(pos: Vec3) -> Self {
        Self {
            rep: EdgePtr(None),
            pos,
            aux: Default::default(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Face<T> {
    pub rep: EdgePtr,

    pub aux: T,
}

// The doubly connected edge list (DCEL or Doconeli), also known as half-edge data structure,
// is a data structure to represent an embedding of a planar graph in the plane, and polytopes in 3D.
#[derive(Default, Clone, Debug)]
pub struct Doconeli<E, V, F> {
    edges: Memory<Edge<E>>,
    vrtxs: Memory<Vrtx<V>>,
    faces: Memory<Face<F>>,

    _boo: PhantomData<Edge<E>>,
}

impl<
        EdgeData: Default + Clone + Debug,
        VrtxData: Default + Clone + Debug,
        FaceData: Default + Clone + Debug,
    > Doconeli<EdgeData, VrtxData, FaceData>
{
    // Initialize an empty DCEL with `n` vertices and `m` faces.
    // pub fn new(n: usize, m: usize) -> Self {
    //     Self {
    //         edges: vec![],
    //         faces: vec![Face::default(); m],
    //         vrtxs: vec![Vrtx::default(); n],
    //         _boo: Default::default(),
    //     }
    // }

    // Initialize an empty DCEL.
    pub fn new() -> Self {
        Self {
            edges: Memory::new(),
            faces: Memory::new(),
            vrtxs: Memory::new(),
            _boo: Default::default(),
        }
    }

    pub fn from_vertices_and_faces(
        vertices: Vec<Vec3>,
        faces: Vec<Vec<usize>>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut mesh = Self::new();

        // https://cs.stackexchange.com/questions/2450/how-do-i-construct-a-doubly-connected-edge-list-given-a-set-of-line-segments

        // Need mapping between original indices, and new pointers
        let vertex_pointers = vec![];
        // Need mapping between vertices and edges
        let mut v_to_e = HashMap::<VrtxPtr, Vec<EdgePtr>>::new();

        // 1. For each endpoint, create a vertex.
        for (vertex_id, vertex_data) in vertices.into_iter().enumerate() {
            let index = VrtxPtr(Some(mesh.vrtxs.alloc(Vrtx::new(vertex_data))));
            vertex_pointers.push(index);
        }

        // 2. For each edge, create two half-edges, and assign their root vertices and twins.
        let edges = faces
            .iter()
            .map(|face| {
                let mut conc = face;
                conc.push(face[0]);

                conc.windows(2)
                    .map(|pair| (vertex_pointers[pair[0]], vertex_pointers[pair[1]]))
            })
            .flatten()
            .collect_vec();

        for (v_a, v_b) in edges {
            // Assign root vertices
            let edge = mesh.edges.alloc(Edge::new(v_a));
            let edge_twin = mesh.edges.alloc(Edge::new(v_b));

            let edge_pointer = EdgePtr(Some(edge));
            let edge_twin_pointer = EdgePtr(Some(edge_twin));

            // Assign twins
            // should have function to add, and remove.. etc. DO NOT USE EDGES DIRECTLY :/
            mesh.edges.deref_mut(edge).twin = edge_twin_pointer;
            mesh.edges.deref_mut(edge_twin).twin = edge_pointer;

            // Add to v_to_e
            v_to_e
                .entry(v_a)
                .or_insert_with(Vec::new)
                .push(edge_pointer);

            v_to_e
                .entry(v_b)
                .or_insert_with(Vec::new)
                .push(edge_twin_pointer);
        }

        // 3. For each endpoint, sort the half-edges whose tail vertex is that endpoint in clockwise order.
        for vertex in mesh.vrtxs.items() {}

        for (face_id, face) in faces.iter().enumerate() {
            // TODO: should instantiate this edge first, then grab its id, then use this id.
            let e0_id = mesh.edges.len();
            mesh.faces[face_id] = Face {
                rep: Some(EdgeID(e0_id)),
                ..default()
            };

            for (i, &vertex_id) in face.iter().enumerate() {
                mesh.edges.push(Edge {
                    root: VrtxID(vertex_id),
                    face: FaceID(face_id),
                    next: Some(EdgeID(e0_id + ((i + 1) % face.len()))),
                    twin: None,
                    ..default()
                });

                if mesh.vrtxs[vertex_id].rep.is_none() {
                    mesh.vrtxs[vertex_id] = Vrtx {
                        rep: Some(EdgeID(e0_id + i)),
                        position: Vec3::new(
                            vertices[vertex_id][0],
                            vertices[vertex_id][1],
                            vertices[vertex_id][2],
                        ),
                        ..default()
                    };
                }
            }
        }

        let mut vertex_pair_to_edge_map = HashMap::new();
        for edge_id in 0..mesh.edges.len() {
            let v_a = mesh.get_root_of_edge(edge_id);
            let v_b = mesh.get_root_of_edge(mesh.get_next_of_edge(edge_id));
            if let Some(&dupl_id) = vertex_pair_to_edge_map.get(&(v_a, v_b)) {
                bail!("NOT MANIFOLD: Duplicate vertex pair ({v_a}, {v_b}) for both edge {dupl_id} and edge {edge_id}.")
            }
            vertex_pair_to_edge_map.insert((v_a, v_b), edge_id);
        }

        for (&(v_a, v_b), &e_ab) in &vertex_pair_to_edge_map {
            if let Some(&e_ba) = vertex_pair_to_edge_map.get(&(v_b, v_a)) {
                match mesh.edges[e_ab].twin {
                    Some(cur) => {
                        bail!("NOT MANIFOLD: Edge {e_ab} has two twins ({cur:?}, {e_ba}).")
                    }
                    None => mesh.edges[e_ab].twin = Some(EdgeID(e_ba)),
                };
            } else {
                bail!("NOT WATERTIGHT: Edge {e_ab} has no twin.");
            }
        }

        Ok(mesh)
    }

    // // Read an STL file from `path`, and construct a DCEL.
    // pub fn from_stl(path: &str) -> Result<Self, Box<dyn Error>> {
    //     let stl = stl_io::read_stl(&mut OpenOptions::new().read(true).open(path)?)?;

    //     let vertices = stl
    //         .vertices
    //         .into_iter()
    //         .map(|v| Vec3::new(v[0], v[1], v[2]))
    //         .collect_vec();

    //     let faces = stl
    //         .faces
    //         .into_iter()
    //         .map(|f| f.vertices.to_vec())
    //         .collect_vec();

    //     Self::from_vertices_and_faces(&vertices, &faces)
    // }

    // // Read an OBJ file from `path`, and construct a DCEL.
    // pub fn from_obj(path: &str) -> Result<Self, Box<dyn Error>> {
    //     let obj = obj::Obj::load(path)?;

    //     let vertices = obj
    //         .data
    //         .position
    //         .into_iter()
    //         .map(|p| Vec3::from(p))
    //         .collect_vec();

    //     let faces = obj
    //         .data
    //         .objects
    //         .into_iter()
    //         .map(|o| o.groups.into_iter().map(|g| g.polys).flatten())
    //         .flatten()
    //         .map(|poly| poly.0.into_iter().map(|index| index.0).collect_vec())
    //         .collect_vec();

    //     Self::from_vertices_and_faces(&vertices, &faces)
    // }
}

// #[derive(Default, Clone, Debug, Serialize, Deserialize)]
// pub struct VertexAuxData {
//     pub original_face_id: usize,
//     pub ordering: Vec<(usize, usize)>,
// }

// #[derive(Default, Clone, Debug, Serialize, Deserialize)]
// pub struct EdgeAuxData {
//     pub label: Option<usize>,
//     pub part_of_path: Option<usize>,
//     pub edges_between: Option<Vec<usize>>,
//     pub edges_between_endpoints: Option<(usize, usize)>,
// }

// #[derive(Default, Clone, Debug, Serialize, Deserialize)]
// pub struct FaceAuxData {
//     pub color: Color,
//     pub dual_position: Option<Vec3>,
//     pub dual_normal: Option<Vec3>,
//     pub original_face: Option<usize>,
// }
