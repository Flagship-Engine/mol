extern crate mol;
use std::path::Path;

#[test]
fn load_basic() {
    let model = match mol::obj::OBJ::from_path(Path::new("tests/cube.obj")) {
        Ok(model) => model,
        Err(err) => panic!("OBJ error: {:?}", err),
    };
    // println!("{:?}", model);
    
    model.flat_iter().for_each(|(v, u, n)| {
        println!("{:?} {:?} {:?}", v, u, n);
    });
}
