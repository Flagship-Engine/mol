

pub mod obj {
    pub struct Triangle {
        pub indices: [[i32; 3]; 3],
        pub smoothing: bool,
    }
    pub struct Quad {
        pub indices: [[i32; 3]; 4],
        pub smoothing: bool,
    }
    pub struct NGon {
        pub indices: Vec<[i32; 3]>,
        pub smoothing: bool,
    }
    
    pub enum Face {
        Tri(Triangle),
        Quad(Quad),
        NGon(NGon),
    }
    
    pub struct Group {
        pub faces: Vec<Face>,
        pub material: Option<String>,
    }
    
    pub struct Object {
        pub name: String,
        pub meshes: Vec<Group>,
    }
    
    pub struct Material {
        // ...
    }
    
    pub struct OBJ {
        pub positions: Vec<f32>,
        pub normals:   Vec<f32>,
        pub uvs:       Vec<f32>,
        
        pub models: Vec<Object>,
        pub materials: Vec<Material>
    }
}
