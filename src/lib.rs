pub mod douconel;

#[cfg(test)]
mod tests {

    use glam::Vec3;
    use serde::{Deserialize, Serialize};
    use crate::douconel::{Douconel, HasNormal, HasPosition};

    #[test]
    fn from_manual() {
        let faces = vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![0, 3, 2]];
        let douconel = Douconel::<(), (), ()>::from_faces(faces);
        assert!(douconel.is_ok());
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 4);
            assert!(douconel.nr_edges() == 6 * 2);
            assert!(douconel.nr_faces() == 4);

            assert!(douconel.verify_correctness().is_ok());
            assert!(douconel.verify_invariants().is_ok());
        }
    }

    #[test]
    fn from_blub_stl() {

        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        struct VertData {
            position: Vec3,
        }

        impl HasPosition for VertData {
            fn position(&self) -> Vec3 {
                self.position
            }
            fn set_position(&mut self, position: Vec3) {
                self.position = position;
            }
        }

        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        struct FaceData {
            normal: Vec3,
        }

        impl HasNormal for FaceData {
            fn normal(&self) -> Vec3 {
                self.normal
            }
            fn set_normal(&mut self, normal: Vec3) {
                self.normal = normal;
            }
        }

        let douconel = Douconel::<VertData, (), FaceData>::from_stl("assets/blub001k.stl");
        assert!(douconel.is_ok());
        if let Ok(douconel) = douconel {
            assert!(douconel.nr_verts() == 945);
            assert!(douconel.nr_edges() == 2829 * 2);
            assert!(douconel.nr_faces() == 1886);

            assert!(douconel.verify_correctness().is_ok());
            assert!(douconel.verify_invariants().is_ok());
        }
    }

    #[test]
    fn from_nefertiti_stl() {

        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        struct VertData {
            position: Vec3,
        }

        impl HasPosition for VertData {
            fn position(&self) -> Vec3 {
                self.position
            }
            fn set_position(&mut self, position: Vec3) {
                self.position = position;
            }
        }

        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        struct FaceData {
            normal: Vec3,
        }

        impl HasNormal for FaceData {
            fn normal(&self) -> Vec3 {
                self.normal
            }
            fn set_normal(&mut self, normal: Vec3) {
                self.normal = normal;
            }
        }

        let douconel = Douconel::<VertData, (), FaceData>::from_stl("assets/nefertiti099k.stl");
        assert!(douconel.is_ok());
        if let Ok(douconel) = douconel {
            assert!(douconel.nr_verts() == 49971);
            assert!(douconel.nr_edges() == 149907 * 2);
            assert!(douconel.nr_faces() == 99938);

            assert!(douconel.verify_correctness().is_ok());
            assert!(douconel.verify_invariants().is_ok());
        }
    }


}
