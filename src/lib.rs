pub mod douconel;
pub mod douconel_extended;
pub mod utils;

#[cfg(test)]
mod tests {

    use crate::{
        douconel::Douconel,
        douconel_extended::{HasColor, HasNormal, HasPosition},
    };
    use bevy::prelude::*;
    use itertools::Itertools;
    use petgraph::{algo::astar, visit::EdgeRef};
    use rand::seq::SliceRandom;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    #[derive(Default, Copy, Clone)]
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

    #[derive(Default, Copy, Clone)]
    struct FaceData {
        normal: Vec3,
        color: Color,
    }

    impl HasNormal for FaceData {
        fn normal(&self) -> Vec3 {
            self.normal
        }
        fn set_normal(&mut self, normal: Vec3) {
            self.normal = normal;
        }
    }

    impl HasColor for FaceData {
        fn color(&self) -> Color {
            self.color
        }

        fn set_color(&mut self, color: Color) {
            self.color = color;
        }
    }

    #[test]
    fn from_manual() {
        let faces = vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![0, 3, 2]];
        let douconel = Douconel::<(), (), ()>::from_faces(faces);
        assert!(douconel.is_ok());
        if let Ok((douconel, _, _)) = douconel {
            assert!(douconel.nr_verts() == 4);
            assert!(douconel.nr_edges() == 6 * 2);
            assert!(douconel.nr_faces() == 4);

            assert!(douconel.verify_properties().is_ok());
            assert!(douconel.verify_references().is_ok());
            assert!(douconel.verify_invariants().is_ok());

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }
        }
    }

    #[test]
    fn from_blub_stl() {
        let douconel = Douconel::<VertData, (), FaceData>::from_stl("assets/blub001k.stl");
        assert!(douconel.is_ok());
        if let Ok(douconel) = douconel {
            assert!(douconel.nr_verts() == 945);
            assert!(douconel.nr_edges() == 2829 * 2);
            assert!(douconel.nr_faces() == 1886);

            assert!(douconel.verify_properties().is_ok());
            assert!(douconel.verify_references().is_ok());
            assert!(douconel.verify_invariants().is_ok());

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }

            let g = douconel.graph();

            assert!(g.node_count() == 945);
            assert!(g.edge_count() == 2829 * 2);

            let verts = douconel.verts.keys().collect_vec();

            (0..100000).into_par_iter().for_each(|_| {
                let mut rng = rand::thread_rng();
                let (v_a, v_b) = verts
                    .choose_multiple(&mut rng, 2)
                    .copied()
                    .collect_tuple()
                    .unwrap();

                let _path = astar(
                    &g,
                    v_a,
                    |finish| finish == v_b,
                    |e| *e.weight(),
                    |v_id| douconel.distance(v_b, v_id),
                );
            });
        }
    }

    #[test]
    fn from_nefertiti_stl() {
        let douconel = Douconel::<VertData, (), FaceData>::from_stl("assets/nefertiti099k.stl");
        assert!(douconel.is_ok());
        if let Ok(douconel) = douconel {
            assert!(douconel.nr_verts() == 49971);
            assert!(douconel.nr_edges() == 149907 * 2);
            assert!(douconel.nr_faces() == 99938);

            assert!(douconel.verify_properties().is_ok());
            assert!(douconel.verify_references().is_ok());
            assert!(douconel.verify_invariants().is_ok());

            for face_id in douconel.faces.keys() {
                assert!(douconel.corners(face_id).len() == 3);
            }

            let g = douconel.graph();

            assert!(g.node_count() == 49971);
            assert!(g.edge_count() == 149907 * 2);

            let verts = douconel.verts.keys().collect_vec();

            (0..1000).into_par_iter().for_each(|_| {
                let mut rng = rand::thread_rng();
                let (v_a, v_b) = verts
                    .choose_multiple(&mut rng, 2)
                    .copied()
                    .collect_tuple()
                    .unwrap();

                let _path = astar(
                    &g,
                    v_a,
                    |finish| finish == v_b,
                    |e| *e.weight(),
                    |v_id| douconel.distance(v_b, v_id),
                );
            });
        }
    }
}
