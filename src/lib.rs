pub mod obj {
    use {
        std::fs::File,
        std::io::{BufRead, BufReader},
        std::path::Path,
        std::str::FromStr,
    };


    //TODO: perhaps save line number for debugging OBJ files
    #[derive(Debug)]
    pub enum Error {
        IO(std::io::Error),
        ParseObject,
        ParseGroup,
        ParseFace,
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

    impl Group {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_owned(),
                faces: Vec::new(),
                material: None
            }
        }
    }

    #[derive(Debug)]
    pub struct Object {
        pub name: String,
        pub groups: Vec<Group>,
    }

    impl Object {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_owned(),
                groups: Vec::new(),
            }
        }

        /// Returns the last group or a new unnamed group
        fn last_group(&mut self) -> &mut Group {
            if self.groups.is_empty() {
                self.groups.push(Group::new(""));
            }
            self.groups.last_mut().unwrap()
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

        /// Returns the last object created or a new unnamed object
        fn last_object(&mut self) -> &mut Object {
            if self.objects.is_empty() {
                self.objects.push(Object::new(""));
            }
            self.objects.last_mut().unwrap()
        }

        pub fn from_path(path: &Path) -> Result<Self, Error> {
            let mut ret = Self::default();
            let file = File::open(path).map_err(Error::IO)?;
            let reader = BufReader::new(file);

            // Begin parsing of the OBJ

            // Modifies the iterator whether it returns ok or err
            macro_rules! try_take_floats {
                ($count:literal, $iter:expr, $err:expr) => {{
                    let mut ret = [0_f32; $count];
                    for f in &mut ret {
                        *f = $iter
                            .next()
                            .and_then(|word| f32::from_str(word).ok())
                            .ok_or($err)?;
                    }
                    ret
                }};
            }

            // TODO: possibly put this in a different function to allow multiple ways of parsing
            for line in reader.lines() {
                let line = line.map_err(Error::IO)?;
                let mut words = line.split_whitespace();

                match words.next() {
                    // Whichever object is last in the list is where faces are appended to
                    Some("o") => ret
                        .objects
                        .push(Object::new(words.next().ok_or(Error::ParseObject)?)),

                    // Currently this parser will only take the first argument as the group
                    // This may or may not change
                    // Whichever group is last in this list is where faces are appended to
                    Some("g") => ret
                        .last_object().groups
                        .push(Group::new(words.next().ok_or(Error::ParseGroup)?)),

                    Some("v") => ret
                        .positions
                        .extend(&try_take_floats!(3, words, Error::ParseVertex)),
                    Some("vt") => ret
                        .uvs
                        .extend(&try_take_floats!(2, words, Error::ParseTexcoord)),
                    Some("vn") => ret
                        .normals
                        .extend(&try_take_floats!(3, words, Error::ParseNormal)),

                    //TODO: Much much much nicer face parsing
                    Some("f") => {
                        let pos_size  = ret.positions.len();
                        let uv_size   = ret.uvs.len();
                        let norm_size = ret.normals.len();
                        ret.last_object().last_group()
                            .faces.push(parse_face(pos_size, uv_size, norm_size, &mut words)?);
                    },

                    _ => (),
                }
            }
            Ok(ret)
        }
    }

    fn parse_face(pos_size: usize, uv_size: usize, norm_size: usize, words: &mut std::str::SplitWhitespace) -> Result<Face, Error> {
        //TODO: avoid unneeded allocations
        let indices: Vec<Result<[usize; 3], ()>> = words.map(|word| {
            let mut ret = [0_usize; 3];
            let mut iter = word.split('/');
            let pos = i32::from_str(iter.next().ok_or(())?).map_err(|_|())?;
            ret[0] = if pos < 0 {
                pos_size as i32 + pos + 1
            } else {
                pos
            } as usize;

            let coord = match iter.next() {
                //No texture coordinate
                Some("") => 0,
                Some(coord) => i32::from_str(coord).map_err(|_|())?,
                None => 0,
            };
            ret[1] = if coord < 0 {
                uv_size as i32 + coord + 1
            } else {
                coord
            } as usize;
            
            let normal = match iter.next() {
                Some(normal) => i32::from_str(normal).map_err(|_|())?,
                None => 0,
            };
            ret[2] = if normal < 0 {
                norm_size as i32 + normal + 1
            } else {
                normal
            } as usize;
            
            Ok(ret)
        }).collect();
        
        let mut temp = Vec::new();
        
        for int in indices.iter() {
            let ints = int.map_err(|_|Error::ParseFace)?;
            temp.push(ints);
        }
        
        match temp.len() {
            3 => Ok(Face::Tri([temp[0], temp[1], temp[2]])),
            4 => Ok(Face::Quad([temp[0], temp[1], temp[2], temp[3]])),
            x if x > 4 => Ok(Face::NGon(temp)),
            _ => Err(Error::ParseFace)
        }
    }
}
