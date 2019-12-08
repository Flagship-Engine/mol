pub mod obj {
    use {
        std::fs::File,
        std::io::{BufRead, BufReader},
        std::path::Path,
        std::str::FromStr,
    };

    #[derive(Debug)]
    pub enum Error {
        Load(std::io::Error),
        ParseObject,
        ParseVertex,
        ParseNormal,
        ParseTexcoord,
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
            Self {
                name: name.to_owned(),
                meshes: Vec::new(),
            }
        }
    }

    #[derive(Debug)]
    pub struct Material {
        pub name: String,
    }

    #[derive(Debug, Default)]
    pub struct OBJ {
        pub positions: Vec<f32>,
        pub normals: Vec<f32>,
        pub uvs: Vec<f32>,

        pub objects: Vec<Object>,
        pub materials: Vec<Material>,
    }

    impl OBJ {
        /// Creates and empty OBJ with reserved capacity for vertices
        pub fn new(capacity: usize) -> Self {
            Self {
                positions: Vec::with_capacity(capacity),
                normals: Vec::with_capacity(capacity),
                uvs: Vec::with_capacity(capacity),
                objects: Vec::with_capacity(1),
                materials: Vec::new(),
            }
        }

        pub fn from_path(path: &Path) -> Result<Self, Error> {
            let mut ret = Self::default();
            let file = File::open(path).map_err(Error::Load)?;
            let reader = BufReader::new(file);

            // Begin parsing of the OBJ

            // Modifies the iterator whether it returns ok or err
            macro_rules! try_take_floats {
                ($count:literal, $iter:expr, $err:expr) => {{
                    let mut ret = [0f32; $count];
                    for f in &mut ret {
                        *f = $iter
                            .next()
                            .and_then(|word| f32::from_str(word).ok())
                            .ok_or($err)?;
                    }
                    ret
                }};
            }

            //TODO: possibly put this in a different function to allow multiple ways of parsing
            for line in reader.lines() {
                let line = line.map_err(Error::Load)?;
                let mut words = line.split_whitespace();

                match words.next() {
                    // Whichever object is last in the list is where faces are appended to
                    Some("o") => ret
                        .objects
                        .push(Object::new(words.next().ok_or(Error::ParseObject)?)),
                    Some("v") => ret
                        .positions
                        .extend(&try_take_floats!(3, words, Error::ParseVertex)),
                    Some("vt") => ret
                        .uvs
                        .extend(&try_take_floats!(2, words, Error::ParseTexcoord)),
                    Some("vn") => ret
                        .normals
                        .extend(&try_take_floats!(3, words, Error::ParseNormal)),
                    _ => (),
                }
            }
            Ok(ret)
        }
    }
}
