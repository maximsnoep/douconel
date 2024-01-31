pub mod douconel;
pub mod mem;

#[cfg(test)]
mod tests {
    use crate::douconel::Doconeli;

    #[test]
    fn from_list() {
        let douconel = Doconeli::<(), (), ()>::new();

        for i in 0..10 {
            douconel.push_back(i);
        }
        println!("");
        douconel.debug_print();

        for i in 10..20 {
            douconel.push_front(i);
        }
        println!("");
        douconel.debug_print();
    }

    // #[test]
    // fn from_stl() {
    //     let douconel = Doconeli::<(), (), ()>::from_stl("assets/blub001k.stl");
    //     assert!(douconel.is_ok());
    //     assert!(douconel.as_ref().unwrap().vertices.len() == 945);
    //     assert!(douconel.as_ref().unwrap().edges.len() == 2829 * 2);
    //     assert!(douconel.as_ref().unwrap().faces.len() == 1886);
    // }

    // #[test]
    // fn from_obj() {
    //     let douconel = Doconeli::<(), (), ()>::from_obj("assets/blub001k.obj");
    //     assert!(douconel.is_ok());
    //     assert!(douconel.as_ref().unwrap().vertices.len() == 945);
    //     assert!(douconel.as_ref().unwrap().edges.len() == 2829 * 2);
    //     assert!(douconel.as_ref().unwrap().faces.len() == 1886);
    // }
}
