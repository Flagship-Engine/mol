
pub mod obj {
    use {
        std::io::{ prelude::*, BufReader },
        std::fs::File,
        std::path::Path,
        std::str::FromStr,
    };
    
    #[derive(Debug)]
    pub enum OBJError {
        Load(std::io::Error),
        ParseObject(),
        ParseVertex(),
        ParseNormal(),
        ParseTexcoord(),
    }
    
    #[derive(Debug)]
    pub enum Face {
        Tri([[usize; 3]; 3]),
        Quad([[usize; 3]; 4]),
        NGon(Vec<[usize; 3]>),
    }
    
    #[derive(Debug)]
    pub struct Group {
        pub name: String,
        pub faces: Vec<Face>,
        pub material: Option<usize>,
    }
    
    #[derive(Debug)]
    pub struct Object {
        pub name: String,
        pub meshes: Vec<Group>,
    }
    
    impl Object {
        pub fn new(name: &str) -> Self {
            Object {
                name: name.to_owned(),
                meshes: Vec::new(),
            }
        }
    }
    
    #[derive(Debug)]
    pub struct Material {
        pub name: String
    }
    
    #[derive(Debug, Default)]
    pub struct OBJ {
        pub positions: Vec<f32>,
        pub normals:   Vec<f32>,
        pub uvs:       Vec<f32>,
        
        pub objects:   Vec<Object>,
        pub materials: Vec<Material>
    }
    
    impl OBJ {
        ///Creates and empty OBJ with reserved capacity for vertices
        pub fn new(capacity: usize) -> Self {
            OBJ {
                positions: Vec::with_capacity(capacity),
                normals:   Vec::with_capacity(capacity),
                uvs:       Vec::with_capacity(capacity),
                objects:   Vec::with_capacity(1),
                materials: Vec::new()
            }
        }
        pub fn from_path(path: &Path) -> Result<Self, OBJError> {
            let mut ret = OBJ::default();
            
            let file = File::open(path).map_err(OBJError::Load)?;
            let reader = BufReader::new(file);
            
            //Begin parsing of the OBJ
            //TODO: possibly put this in a different function to allow multiple ways of parsing
            for line in reader.lines() {
                let line = line.map_err(OBJError::Load)?;
                let mut words = line.split_whitespace();
                
                match words.next() {
                    //Object
                    Some("o") => {
                        if let Some(name) = words.next() {
                            //Whichever object is last in the list is where faces are appended to
                            ret.objects.push(Object::new(name));
                        } else {
                            return Err(OBJError::ParseObject());
                        }
                    },
                    
                    //Vertex
                    Some("v") => {
                        let floats = take3(&mut words).map_err(|_| OBJError::ParseVertex())?;
                        floats.iter().for_each(|f| ret.positions.push(*f));
                    },
                    Some("vt") => {
                        let floats = take2(&mut words).map_err(|_| OBJError::ParseNormal())?;
                        floats.iter().for_each(|f| ret.positions.push(*f));
                    },
                    Some("vn") => {
                        let floats = take3(&mut words).map_err(|_| OBJError::ParseNormal())?;
                        floats.iter().for_each(|f| ret.positions.push(*f));
                    },
                    
                    _ => {},
                };
            }
            
            Ok(ret)
        }
    }
    
    //Modifies the iterator whether it returns ok or err
    #[inline(always)]
    fn take3(iter: &mut std::str::SplitWhitespace<'_>) -> Result<[f32; 3], ()> {
        let mut ret = [0f32; 3];
        
        for f in &mut ret {
            if let Some(word) = iter.next() {
                if let Ok(float) = f32::from_str(word) {
                    *f = float;
                    continue;
                }
            }
            return Err(());
        }
        
        Ok(ret)
    }
    #[inline(always)]
    fn take2(iter: &mut std::str::SplitWhitespace<'_>) -> Result<[f32; 2], ()> {
        let mut ret = [0f32; 2];
        
        for f in &mut ret {
            if let Some(word) = iter.next() {
                if let Ok(float) = f32::from_str(word) {
                    *f = float;
                    continue;
                }
            }
            return Err(());
        }
        
        Ok(ret)
    }
}
