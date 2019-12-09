use mol;
use mol::{Vec3, Vec2};
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

#[test]
fn load_cube_vn() {
    let model = match mol::obj::OBJ::from_path(Path::new("tests/cube_vn.obj")) {
        Ok(model) => model,
        Err(err) => panic!("OBJ error: {:?}", err),
    };
    let expected = [
        (Vec3::new(1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(-1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(-1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        (Vec3::new(1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        (Vec3::new(-1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        (Vec3::new(-1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        (Vec3::new(-1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        (Vec3::new(-1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        (Vec3::new(-1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        (Vec3::new(-1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        (Vec3::new(-1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        (Vec3::new(1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        (Vec3::new(1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        (Vec3::new(-1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        (Vec3::new(1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(1.0, 1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(1.0, -1.0, 1.0), Vec2::new(0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        (Vec3::new(-1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(-1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(1.0, 1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(1.0, -1.0, -1.0), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
    ];

    model
        .flat_iter()
        .zip(expected.iter())
        .enumerate()
        .for_each(|(i, (a, e))| assert_eq!(&a, e, "(output index {})", i));
}
