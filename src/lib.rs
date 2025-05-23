#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
pub mod douconel;
pub mod douconel_bevy;
pub mod douconel_embedded;
pub mod douconel_io;
pub mod douconel_petgraph;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        douconel::{Douconel, Empty},
        douconel_embedded::EmbeddedVertex,
    };

    slotmap::new_key_type! {
        struct VertID;
        struct EdgeID;
        struct FaceID;
    }

    #[test]
    fn from_manual() {
        let faces = vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![0, 3, 2]];
        let douconel = Douconel::<VertID, Empty, EdgeID, Empty, FaceID, Empty>::from_faces(&faces);
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 4);
            assert!(douconel.nr_edges() == 6 * 2);
            assert!(douconel.nr_faces() == 4);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn from_blub_stl() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/blub001k.stl"));
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 945);
            assert!(douconel.nr_edges() == 2829 * 2);
            assert!(douconel.nr_faces() == 1886);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn from_blub_obj() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/blub001k.obj"));
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 945);
            assert!(douconel.nr_edges() == 2829 * 2);
            assert!(douconel.nr_faces() == 1886);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn from_nefertiti_stl() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/nefertiti099k.stl"));
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 49971);
            assert!(douconel.nr_edges() == 149_907 * 2);
            assert!(douconel.nr_faces() == 99938);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn from_hexahedron_obj() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/hexahedron.obj"));
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 8);
            assert!(douconel.nr_edges() == 4 * 6);
            assert!(douconel.nr_faces() == 6);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 4);
            }
        }
    }

    #[test]
    fn from_tetrahedron_obj() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/tetrahedron.obj"));
        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 4);
            assert!(douconel.nr_edges() == 3 * 4);
            assert!(douconel.nr_faces() == 4);

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn serialize() {
        let douconel = Douconel::<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>::from_file(&PathBuf::from("assets/nefertiti099k.stl"));

        assert!(douconel.is_ok(), "{douconel:?}");
        if let Ok((douconel, _, _)) = douconel {
            let serialized = serde_json::to_string(&douconel);
            assert!(serialized.is_ok(), "{:?}", serialized.unwrap());

            if let Ok(serialized) = serialized {
                let deserialized = serde_json::from_str::<Douconel<VertID, EmbeddedVertex, EdgeID, Empty, FaceID, Empty>>(&serialized);

                assert!(deserialized.is_ok(), "{deserialized:?}");
                if let Ok(deserialized) = deserialized {
                    assert!(douconel.nr_verts() == deserialized.nr_verts());
                    assert!(douconel.nr_edges() == deserialized.nr_edges());
                    assert!(douconel.nr_faces() == deserialized.nr_faces());
                }
            }
        }
    }
}
