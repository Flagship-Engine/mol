//! A self-contained 3D model parser/loader, written entirely in Rust.
//! Only Wavefront OBJ models are supported currently, but this will be expanded upon in the future.
//!
//!# Example usage
//! ```
//! use mol::obj;
//! use std::path::Path;
//!
//! let model = obj::OBJ::from_path(Path::new("tests/cube.obj"));
//!
//! match model {
//!     Ok(model) => {
//!         //Loaded the OBJ successfully!
//!         //Printing all objects and groups in an OBJ file
//!         for object in model.objects.iter() {
//!             println!("Object: {}", object.name);
//!             for group in object.groups.iter() {
//!                 println!("  Group: {}", group.name);
//!             }
//!         }
//!         //Printing all vertices in an OBJ file
//!         for (pos, uv, norm) in model.flat_iter() {
//!             println!("Vertex position: {}", pos);
//!             println!("Vertex texture coordinate: {}", uv);
//!             println!("Vertex normal: {}", norm);
//!         }
//!     },
//!     Err(err) => panic!("Failed to load OBJ: {:?}", err),
//! }
//! ```

pub mod math;
pub mod triangulate;

/// Everything related to the Wavefront OBJ file format.
pub mod obj {
    use {
        crate::math::{Vec2, Vec3},
        std::fs::File,
        std::io::{BufRead, BufReader},
        std::path::Path,
        std::str::FromStr,
    };

    pub type Vertex = [usize; 3];

    //TODO: perhaps save line number for debugging OBJ files
    /// Errors that may occur during [`OBJ`] parsing.
    ///
    /// [`OBJ`]: obj/struct.OBJ.html
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

    /// Represents a single face of an [`Object`].
    ///
    /// [`Object`]: struct.Object.html
    #[derive(Debug)]
    pub enum Face {
        // Triangulated face
        Tri([Vertex; 3]),
        // Non-triangulated quad
        Quad([Vertex; 4]),
        // Non-triangulated polygon
        NGon(Vec<Vertex>),
    }

    impl Face {
        fn get_vertex(&self, index: usize) -> Option<&Vertex> {
            match self {
                Self::Tri(ints) => ints.get(index),
                Self::Quad(ints) => ints.get(index),
                Self::NGon(ints) => ints.get(index),
            }
        }
    }

    /// Represents a group within an [`Object`].
    ///
    /// [`Object`]: struct.Object.html
    #[derive(Debug)]
    pub struct Group {
        /// name of the group
        pub name: String,
        /// the [`Face`]s of the group
        ///
        /// [`Face`]: struct.Face.html
        pub faces: Vec<Face>,
        /// optional [`Material`] id
        ///
        /// [`Material`]: struct.Material.html
        pub material: Option<usize>,
    }

    impl Group {
        /// Constructs an empty group with the given name.
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_owned(),
                faces: Vec::new(),
                material: None,
            }
        }
    }

    /// Represents an object within an [`OBJ`].
    ///
    /// [`OBJ`]: struct.OBJ.html
    #[derive(Debug)]
    pub struct Object {
        /// name of the object
        pub name: String,
        /// [`Group`]s contained in the object
        ///
        /// [`Group`]: struct.Group.html
        pub groups: Vec<Group>,
    }

    impl Object {
        /// Constructs an empty object with a given name.
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_owned(),
                groups: Vec::new(),
            }
        }

        /// Returns the last [`Group`] in the object, creating a new default
        /// group if needed.
        ///
        /// [`Group`]: struct.Group.html
        fn last_group(&mut self) -> &mut Group {
            if self.groups.is_empty() {
                self.groups.push(Group::new("default"));
            }
            self.groups.last_mut().unwrap()
        }
    }

    /// Represents a material in an mtl file.
    #[derive(Debug)]
    pub struct Material {
        pub name: String,
    }

    /// Represents the content of an OBJ file.
    #[derive(Debug, Default)]
    pub struct OBJ {
        /// vertex positions
        pub positions: Vec<Vec3>,
        /// vertex normals
        pub normals: Vec<Vec3>,
        /// texture coordinates
        pub uvs: Vec<Vec2>,

        /// [`Object`]s contained in the OBJ
        ///
        /// [`Object`]: struct.Object.html
        pub objects: Vec<Object>,
        /// [`Material`]s loaded from material libraries provided with the `mtllib` directive
        ///
        /// [`Material`]: struct.Material.html
        pub materials: Vec<Material>,
    }

    impl OBJ {
        /// Constructs an empty OBJ.
        pub fn new() -> Self {
            Self {
                positions: Vec::new(),
                normals: Vec::new(),
                uvs: Vec::new(),
                objects: Vec::new(),
                materials: Vec::new(),
            }
        }

        /// Constructs an empty OBJ with reserved capacity for vertices,
        /// normals, and uvs.
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                positions: Vec::with_capacity(capacity),
                normals: Vec::with_capacity(capacity),
                uvs: Vec::with_capacity(capacity),
                objects: Vec::with_capacity(1),
                materials: Vec::new(),
            }
        }

        /// Creates an iterator over all the vertex information stored in the
        /// `OBJ`, obtained by flattening the internal hierarchy of [`Object`]s
        /// and [`Group`]s.
        ///
        /// This method provides a convenient way of processing all the vertex
        /// information of a given `OBJ`.
        ///
        /// Combined with [`map`], this iterator can be used to access just the
        /// part of the vertex information needed for a given situation, such as
        /// just vertex coordinates, normals, or texture coordinates.
        ///
        ///# Examples
        /// ```
        /// use mol::obj::*;
        /// use std::path::Path;
        ///
        /// let obj = OBJ::from_path(Path::new("tests/cube.obj")).unwrap();
        ///
        /// //Printing all vertices in an OBJ file
        /// for (pos, uv, norm) in obj.flat_iter() {
        ///     println!("Vertex position: {}", pos);
        ///     println!("Vertex texture coordinate: {}", uv);
        ///     println!("Vertex normal: {}", norm);
        /// }
        /// ```
        /// Making use of [`map`] to ignore unnecessary data:
        /// ```
        ///# use mol::obj::*;
        ///# use std::path::Path;
        /// let obj = OBJ::from_path(Path::new("tests/cube.obj")).unwrap();
        ///
        /// for normal in obj.flat_iter().map(|(_, _, norm)| norm) {
        ///     // ...
        /// }
        /// ```
        ///
        /// [`OBJ`]: struct.OBJ.html
        /// [`flat_iter`]: struct.OBJ.html#method.flat_iter
        /// [`Object`]: struct.Object.html
        /// [`Group`]: struct.Group.html
        /// [`map`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map
        pub fn flat_iter(&self) -> FlatIterator {
            FlatIterator {
                obj: &self,
                object: 0,
                group: 0,
                face: 0,
                vertex: 0,
            }
        }

        /// Returns the last [`Object`] created, creating a default if needed.
        ///
        /// [`Object`]: struct.Object.html
        fn last_object(&mut self) -> &mut Object {
            if self.objects.is_empty() {
                self.objects.push(Object::new("default"));
            }
            self.objects.last_mut().unwrap()
        }

        /// Constructs an `OBJ` based on the content of the file referenced by
        /// the given [`Path`].
        ///
        /// If an error occurs during loading or parsing of the `OBJ`, a
        /// [`Result`]`::Err` containing the given [`obj::Error`] is returned.
        /// Otherwise, a `Result::Ok` containing the resulting `OBJ` is
        /// returned.
        ///
        /// [`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
        /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
        /// [`obj::Error`]: enum.Error.html
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
                        .push(try_take_floats!(3, words, Error::ParseVertex).into()),
                    Some("vt") => ret
                        .uvs
                        .push(try_take_floats!(2, words, Error::ParseTexcoord).into()),
                    Some("vn") => ret
                        .normals
                        .push(try_take_floats!(3, words, Error::ParseNormal).into()),

                    Some("f") => {
                        let pos_size = ret.positions.len();
                        let uv_size = ret.uvs.len();
                        let norm_size = ret.normals.len();
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
            .collect::<Result<Vec<Vertex>, Error>>()?;

        match indices.len() {
            0..=2 => Err(Error::ParseFace),
            3 => Ok(Face::Tri([indices[0], indices[1], indices[2]])),
            4 => Ok(Face::Quad([indices[0], indices[1], indices[2], indices[3]])),
            _ => Ok(Face::NGon(indices)),
        }
    }

    /// An iterator over all the vertex information stored in an [`OBJ`],
    /// obtained by flattening the internal hierarchy of [`Object`]s and [`Group`]s.
    ///
    /// `FlatIterators` are created by the [`flat_iter`] method on [`OBJ`].
    /// See its documentation for more information.
    ///
    /// [`OBJ`]: struct.OBJ.html
    /// [`flat_iter`]: struct.OBJ.html#method.flat_iter
    /// [`Object`]: struct.Object.html
    /// [`Group`]: struct.Group.html
    pub struct FlatIterator<'a> {
        obj: &'a OBJ,
        object: usize,
        group: usize,
        face: usize,
        vertex: usize,
    }
    impl Iterator for FlatIterator<'_> {
        // Position, coordinate, normal
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
                            Vec2::new(0.0, 0.0)
                        };
                        let norms = if vertex[2] > 0 {
                            self.obj.normals[vertex[2] - 1]
                        } else {
                            Vec3::new(0.0, 0.0, 0.0)
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
