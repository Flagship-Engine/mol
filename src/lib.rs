#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl From<[f32; 3]> for Vec3 {
    fn from(array: [f32; 3]) -> Self {
        Vec3 { x: array[0], y: array[1], z: array[2] }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl From<[f32; 2]> for Vec2 {
    fn from(array: [f32; 2]) -> Self {
        Vec2 { x: array[0], y: array[1] }
    }
}

pub mod obj {
    use {
        std::fs::File,
        std::io::{BufRead, BufReader},
        std::path::Path,
        std::str::FromStr,
        
        super::{Vec3, Vec2}
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

    impl Face {
        fn get_vertex(&self, index: usize) -> Option<&[usize; 3]> {
            match self {
                Self::Tri(ints) => ints.get(index),
                Self::Quad(ints) => ints.get(index),
                Self::NGon(ints) => ints.get(index),
            }
        }
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
                material: None,
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

        /// Returns the last group or a new default group
        fn last_group(&mut self) -> &mut Group {
            if self.groups.is_empty() {
                self.groups.push(Group::new("default"));
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
        pub positions: Vec<Vec3>,
        pub normals: Vec<Vec3>,
        pub uvs: Vec<Vec2>,

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

        pub fn flat_iter(&self) -> FlatIterator {
            FlatIterator {
                obj: &self,
                object: 0,
                group: 0,
                face: 0,
                vertex: 0,
            }
        }

        /// Returns the last object created or a new default object
        fn last_object(&mut self) -> &mut Object {
            if self.objects.is_empty() {
                self.objects.push(Object::new("default"));
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
                        .last_object()
                        .groups
                        .push(Group::new(words.next().ok_or(Error::ParseGroup)?)),

                    Some("v") => ret
                        .positions
                        .push(Vec3::from(try_take_floats!(3, words, Error::ParseVertex))),
                    Some("vt") => ret
                        .uvs
                        .push(Vec2::from(try_take_floats!(2, words, Error::ParseTexcoord))),
                    Some("vn") => ret
                        .normals
                        .push(Vec3::from(try_take_floats!(3, words, Error::ParseNormal))),

                    Some("f") => {
                        let pos_size = ret.positions.len() / 3;
                        let uv_size = ret.uvs.len() / 2;
                        let norm_size = ret.normals.len() / 3;
                        ret.last_object()
                            .last_group()
                            .faces
                            .push(parse_face(pos_size, uv_size, norm_size, &mut words)?);
                    }

                    _ => (),
                }
            }
            Ok(ret)
        }
    }

    fn parse_face(
        pos_size: usize,
        uv_size: usize,
        norm_size: usize,
        words: &mut std::str::SplitWhitespace,
    ) -> Result<Face, Error> {
        macro_rules! calc_index {
            ($index:expr, $count:expr) => {
                if $index < 0 {
                    $count as i32 + $index + 1
                } else {
                    $index
                } as usize
            };
        }

        let indices = words
            .map(|word| {
                let mut iter = word.split('/');
                let pos = i32::from_str(iter.next().ok_or(Error::ParseFace)?)
                    .map_err(|_| Error::ParseFace)?;
                let coord = match iter.next() {
                    None | Some("") => 0,
                    Some(coord) => i32::from_str(coord).map_err(|_| Error::ParseFace)?,
                };
                let normal = match iter.next() {
                    Some(normal) => i32::from_str(normal).map_err(|_| Error::ParseFace)?,
                    None => 0,
                };
                Ok([
                    calc_index!(pos, pos_size),
                    calc_index!(coord, uv_size),
                    calc_index!(normal, norm_size),
                ])
            })
            .collect::<Result<Vec<[usize; 3]>, Error>>()?;

        match indices.len() {
            0..=2 => Err(Error::ParseFace),
            3 => Ok(Face::Tri([indices[0], indices[1], indices[2]])),
            4 => Ok(Face::Quad([indices[0], indices[1], indices[2], indices[3]])),
            _ => Ok(Face::NGon(indices)),
        }
    }

    pub struct FlatIterator<'a> {
        obj: &'a OBJ,
        object: usize,
        group: usize,
        face: usize,
        vertex: usize,
    }
    impl Iterator for FlatIterator<'_> {
        type Item = (Vec3, Vec2, Vec3);

        fn next(&mut self) -> Option<Self::Item> {
            let object = self.obj.objects.get(self.object)?;
            if let Some(group) = object.groups.get(self.group) {
                if let Some(face) = group.faces.get(self.face) {
                    if let Some(vertex) = face.get_vertex(self.vertex) {
                        self.vertex += 1;
                        let pos = self.obj.positions[vertex[0] - 1];
                        let uvs = if vertex[1] > 0 {
                            self.obj.uvs[vertex[1] - 1]
                        } else {
                            Vec2::from([0.0, 0.0])
                        };
                        let norms = if vertex[2] > 0 {
                            self.obj.normals[vertex[2] - 1]
                        } else {
                            Vec3::from([0.0, 0.0, 0.0])
                        };
                        return Some((pos, uvs, norms));
                    } else {
                        self.vertex = 0;
                        self.face += 1;
                    }
                } else {
                    self.face = 0;
                    self.group += 1;
                }
            } else {
                self.group = 0;
                self.object += 1;
            }
            self.next()
        }
    }
}
