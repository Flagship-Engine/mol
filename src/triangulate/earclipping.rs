//! Everything related to triangulation using ear clipping.

use crate::math::{Mat3, Vec3};
use crate::obj::{Face, Group, Object, Vertex, OBJ};

macro_rules! approx_eq {
    ($a:expr, $b:expr) => {
        ($a - $b).abs() < std::f32::EPSILON
    };
}

macro_rules! cond {
    ($test:expr => $then:expr; $else:expr ) => {
        if $test {
            $then
        } else {
            $else
        };
    };
}

/// false = clockwise,
/// true = counter-clockwise
type Winding = bool;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

struct Triangle([Point; 3]);

impl Triangle {
    /// Checks whether the given point is contained withing the triangle.
    fn contains(&self, p: Point) -> bool {
        let Triangle([a, b, c]) = self;

        macro_rules! det {
            ($s:expr, $t:expr, $u:expr) => {
                ($t.y - $u.y) * ($s.x - c.x) + ($u.x - $t.x) * ($s.y - c.y)
            };
        }

        let det = det!(a, b, c);
        if approx_eq!(det, 0.0) {
            return true;
        }
        let inv_det = 1.0 / det;

        let l1 = inv_det * det!(p, b, c);
        if l1 < 0.0 {
            return false;
        }

        let l2 = inv_det * det!(p, c, a);
        if l2 < 0.0 {
            return false;
        }

        l1 + l2 < 1.0
    }

    #[inline(always)]
    /// Determines the winding order of the triangle.
    fn winding(&self) -> Winding {
        let Triangle([a, b, c]) = self;
        (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y) > 0.0
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// Errors related to triangulation using ear clipping.
pub enum Error {
    TooFewVertices,
    NoEarFound,
    NoValidFacesFound,
    NoValidGroupsFound,
    NoValidObjectsFound,
}

impl OBJ {
    #[inline(always)]
    /// Returns the 3D position of the given vertex.
    fn pos(&self, vertex: Vertex) -> Vec3 {
        self.positions[vertex[0] - 1]
    }

    #[inline(always)]
    /// Calculates the 2D projected point of a vertex, based on a
    /// transformation matrix.
    fn point(&self, vertex: Vertex, tf: &Mat3) -> Point {
        let Vec3 { x, z: y, .. } = tf * self.pos(vertex);
        Point { x, y }
    }
}

#[inline(always)]
/// Determines the winding order of the 2D projection of a given polygon,
/// based on a transformation matrix.
fn polygon_winding(polygon: &[Vertex], obj: &OBJ, tf: &Mat3) -> Winding {
    let mut index = 0;
    let mut left = obj.point(polygon[index], tf);
    for (i, vertex) in polygon.iter().enumerate() {
        let point = obj.point(*vertex, tf);
        if point.x < left.x || (approx_eq!(point.x, left.x) && point.y < left.y) {
            index = i;
            left = point;
        }
    }

    let max_index = polygon.len() - 1;
    Triangle([
        obj.point(polygon[cond!(index > 0 => index - 1; max_index)], tf),
        obj.point(polygon[index], tf),
        obj.point(polygon[cond!(index < max_index => index + 1; 0)], tf),
    ])
    .winding()
}

#[derive(Debug)]

/// Iterator for triangulating a [`Face`] of an [`OBJ`].
///
/// [`Face`]: /mol/obj/struct.Face.html
/// [`OBJ`]: /mol/obj/struct.OBJ.html
pub struct FaceTriangulator<'obj> {
    polygon: Vec<Vertex>,
    obj: &'obj OBJ,
    polygon_winding: bool,
    reflex_indices: Vec<usize>,
    tf_flatten: Mat3,
}

impl<'obj> FaceTriangulator<'obj> {
    /// Constructs a `FaceTriangulator` based on an [`OBJ`] and a [`Face`]
    /// thereof.
    ///
    /// If the given face has less than 3 vertices, a [`Result`]`::Err`
    /// containing a corresponding [`earclipping::Error`] is returned.
    /// Otherwise, a `Result::Ok` containing the resulting triangulator is
    /// returned.
    ///
    /// [`OBJ`]: /mol/obj/struct.OBJ.html
    /// [`Face`]: /mol/obj/struct.Face.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`earclipping::Error`]: enum.Error.html
    pub fn new(obj: &'obj OBJ, face: &Face) -> Result<Self> {
        Self::from_obj_polygon(
            obj,
            match face {
                Face::Tri(triangle) => triangle.to_vec(),
                Face::Quad(quad) => quad.to_vec(),
                Face::NGon(vertices) => vertices.clone(),
            },
        )
    }

    fn from_obj_polygon(obj: &'obj OBJ, polygon: Vec<Vertex>) -> Result<Self> {
        if polygon.len() < 3 {
            return Err(Error::TooFewVertices);
        }

        let normal = fit_plane_normal(&obj, &polygon);
        let tf_flatten = Vec3::unit_align(&normal, &Vec3::UNIT_Y);

        Ok(FaceTriangulator {
            polygon_winding: polygon_winding(&polygon, obj, &tf_flatten),
            reflex_indices: vec![],
            polygon,
            obj,
            tf_flatten,
        })
    }
}

impl<'obj> Iterator for FaceTriangulator<'obj> {
    type Item = super::Tri;
    fn next(&mut self) -> Option<Self::Item> {
        if self.polygon.len() < 3 {
            return None;
        }

        let reflex_indices = &mut self.reflex_indices;
        let polygon = &mut self.polygon;
        let obj = &mut self.obj;
        let tf = &self.tf_flatten;

        reflex_indices.clear();
        let max_index = polygon.len() - 1;

        let mut ear = usize::MAX;
        'find_ear: loop {
            ear = ear.wrapping_add(1);
            if ear >= max_index {
                // NOTE: This shouldn't happen, but if it does there's no way to recover, so terminate the iterator
                polygon.clear();
                return None;
            }

            let prev = cond!(ear > 0 => ear - 1; max_index);
            let next = cond!(ear < max_index => ear + 1; 0);
            let tri = Triangle([
                obj.point(polygon[prev], tf),
                obj.point(polygon[ear], tf),
                obj.point(polygon[next], tf),
            ]);

            if tri.winding() != self.polygon_winding {
                reflex_indices.push(ear);
                continue 'find_ear;
            }

            if reflex_indices
                .iter()
                .any(|&j| j != prev && j != next && tri.contains(obj.point(polygon[j], tf)))
            {
                continue 'find_ear;
            }

            if polygon[ear + 1..]
                .iter()
                .map(|vertex| obj.point(*vertex, tf))
                .any(|point| !tri.0.iter().any(|corner| *corner == point) && tri.contains(point))
            {
                continue 'find_ear;
            }

            break 'find_ear;
        }

        let tri = [
            polygon[cond!(ear > 0 => ear - 1 ; max_index)],
            polygon[ear],
            polygon[cond!(ear < max_index => ear + 1 ; 0)],
        ];

        polygon.remove(ear);

        Some(tri)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.polygon.len().saturating_sub(2);
        (count, Some(count))
    }
}

macro_rules! make_triangulation_iterator {
    ($Triangulator: ident {
        $subindex:ident;
        $($where:ident).*;
        $constructor:expr
    }) => {
        impl<'obj> Iterator for $Triangulator<'obj> {
            type Item = super::Tri;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if self.$subindex >= self.$($where).*.len() {
                        return None;
                    }

                    let next = self.triangulator.next();
                    if next.is_some() {
                        return next;
                    }

                    loop {
                        self.$subindex += 1;
                        if self.$subindex >= self.$($where).*.len() {
                            break;
                        }
                        let subtype = &self.$($where).*[self.$subindex];
                        if let Ok(triangulator) = $constructor(self.obj, subtype)
                        {
                            self.triangulator = triangulator;
                            break;
                        }
                    }
                }
            }
        }
    };
}

/// Iterator for triangulating a [`Group`] of an [`OBJ`].
///
/// [`OBJ`]: /mol/obj/struct.OBJ.html
/// [`Group`]: /mol/obj/struct.Group.html
pub struct GroupTriangulator<'obj> {
    obj: &'obj OBJ,
    group: &'obj Group,
    face_index: usize,
    triangulator: FaceTriangulator<'obj>,
}

impl<'obj> GroupTriangulator<'obj> {
    /// Constructs a `GroupTriangulator` based on an [`OBJ`] and a [`Group`]
    /// thereof.
    ///
    /// If the given group has no faces with at least 3 vertices, a
    /// [`Result`]`::Err` containing a corresponding [`earclipping::Error`]
    /// is returned.
    /// Otherwise, a `Result::Ok` containing the resulting triangulator is
    /// returned.
    ///
    /// [`Group`]: /mol/obj/struct.Group.html
    /// [`OBJ`]: /mol/obj/struct.OBJ.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`earclipping::Error`]: enum.Error.html
    pub fn new(obj: &'obj OBJ, group: &'obj Group) -> Result<Self> {
        for face_index in 0..group.faces.len() {
            let face = &group.faces[face_index];
            if let Ok(triangulator) = FaceTriangulator::new(obj, face) {
                return Ok(Self {
                    obj,
                    group: &group,
                    face_index,
                    triangulator,
                });
            }
        }
        Err(Error::NoValidFacesFound)
    }
}

make_triangulation_iterator! {
    GroupTriangulator {
        face_index;
        group.faces;
        FaceTriangulator::new
    }
}

/// Iterator for triangulating an [`Object`] of an [`OBJ`].
///
/// [`OBJ`]: /mol/obj/struct.OBJ.html
/// [`Object`]: /mol/obj/struct.Object.html
pub struct ObjectTriangulator<'obj> {
    obj: &'obj OBJ,
    object: &'obj Object,
    group_index: usize,
    triangulator: GroupTriangulator<'obj>,
}

impl<'obj> ObjectTriangulator<'obj> {
    /// Constructs a `ObjectTriangulator` based on an [`OBJ`] and an [`Object`]
    /// thereof.
    ///
    /// If the given object has no groups with faces with at least 3 vertices,
    /// a [`Result`]`::Err` containing a corresponding [`earclipping::Error`]
    /// is returned.
    /// Otherwise, a `Result::Ok` containing the resulting triangulator is
    /// returned.
    ///
    /// [`OBJ`]: /mol/obj/struct.OBJ.html
    /// [`Object`]: /mol/obj/struct.Object.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`earclipping::Error`]: enum.Error.html
    pub fn new(obj: &'obj OBJ, object: &'obj Object) -> Result<Self> {
        for group_index in 0..object.groups.len() {
            let group = &object.groups[group_index];
            if let Ok(triangulator) = GroupTriangulator::new(obj, group) {
                return Ok(Self {
                    obj,
                    object: &object,
                    group_index,
                    triangulator,
                });
            }
        }
        Err(Error::NoValidGroupsFound)
    }
}

make_triangulation_iterator! {
    ObjectTriangulator {
        group_index;
        object.groups;
        GroupTriangulator::new
    }
}

/// Iterator for triangulating an [`OBJ`].
///
/// [`OBJ`]: /mol/obj/struct.OBJ.html
pub struct ObjTriangulator<'obj> {
    obj: &'obj OBJ,
    object_index: usize,
    triangulator: ObjectTriangulator<'obj>,
}

impl<'obj> ObjTriangulator<'obj> {
    /// Constructs a `ObjTriangulator` based on an [`OBJ`].
    ///
    /// If the given `OBJ` has no objects with groups with faces with at least
    /// 3 vertices, a [`Result`]`::Err` containing a corresponding
    /// [`earclipping::Error`] is returned.
    /// Otherwise, a `Result::Ok` containing the resulting triangulator is
    /// returned.
    ///
    /// [`OBJ`]: /mol/obj/struct.OBJ.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`earclipping::Error`]: enum.Error.html
    pub fn from_obj(obj: &'obj OBJ) -> Result<Self> {
        for object_index in 0..obj.objects.len() {
            let object = &obj.objects[object_index];
            if let Ok(triangulator) = ObjectTriangulator::new(obj, object) {
                return Ok(Self {
                    obj,
                    object_index,
                    triangulator,
                });
            }
        }
        Err(Error::NoValidObjectsFound)
    }
}

make_triangulation_iterator! {
    ObjTriangulator {
        object_index;
        obj.objects;
        ObjectTriangulator::new
    }
}

/// Triangulates the given polygon.
pub fn triangulate(mut polygon: Vec<Vertex>, obj: &OBJ) -> Result<super::Triangulation> {
    match polygon.len() {
        0..=2 => return Err(Error::TooFewVertices),
        3 => return Ok(vec![[polygon[0], polygon[1], polygon[2]]]),
        _ => (),
    };

    let normal = fit_plane_normal(&obj, &polygon);
    let tf = Vec3::unit_align(&normal, &Vec3::UNIT_Y);

    let polygon_winding = polygon_winding(&polygon, obj, &tf);

    let mut triangles = Vec::with_capacity(polygon.len() - 2);
    let mut reflex_indices = Vec::new();

    while polygon.len() >= 3 {
        reflex_indices.clear();

        let max_index = polygon.len() - 1;

        let mut ear = usize::MAX;
        'find_ear: loop {
            ear = ear.wrapping_add(1);
            if ear >= max_index {
                return Err(Error::NoEarFound);
            }

            let prev = cond!(ear > 0 => ear - 1; max_index);
            let next = cond!(ear < max_index => ear + 1; 0);
            let tri = Triangle([
                obj.point(polygon[prev], &tf),
                obj.point(polygon[ear], &tf),
                obj.point(polygon[next], &tf),
            ]);

            if tri.winding() != polygon_winding {
                reflex_indices.push(ear);
                continue 'find_ear;
            }

            if reflex_indices
                .iter()
                .any(|&j| j != prev && j != next && tri.contains(obj.point(polygon[j], &tf)))
            {
                continue 'find_ear;
            }

            if polygon[ear + 1..]
                .iter()
                .map(|vertex| obj.point(*vertex, &tf))
                .any(|point| !tri.0.iter().any(|corner| *corner == point) && tri.contains(point))
            {
                continue 'find_ear;
            }

            break;
        }
        triangles.push([
            polygon[cond!(ear > 0 => ear - 1 ; max_index)],
            polygon[ear],
            polygon[cond!(ear < max_index => ear + 1 ; 0)],
        ]);

        polygon.remove(ear);
    }

    Ok(triangles)
}

/// Calculates the surface normal of the "best fit" plane for the given [vertices],  represented as a [`Vec3`].
///
/// [`Vec3`]: /mol/math/struct.Vec3.html
/// [vertices]: /mol/obj/type.Vertex.html
fn fit_plane_normal(obj: &OBJ, vertices: &[Vertex]) -> Vec3 {
    let mut normal = Vec3::new(0.0, 0.0, 0.0);
    let mut prev = obj.pos(vertices[vertices.len() - 1]);

    for vertex in vertices {
        let curr = obj.pos(*vertex);
        normal.x += (prev.z + curr.z) * (prev.y - curr.y);
        normal.y += (prev.x + curr.x) * (prev.z - curr.z);
        normal.z += (prev.y + curr.y) * (prev.x - curr.x);
        prev = curr;
    }
    normal.normalize();

    normal
}

#[cfg(test)]
mod test {
    use crate::{self as mol, math::Vec3, triangulate::earclipping};

    fn xz_plane_angle_x(x_degrees: f32) -> mol::obj::OBJ {
        let mut obj = mol::obj::OBJ::new();
        let (s, c) = x_degrees.sin_cos();
        macro_rules! make_pos {
            ($x:literal $y:literal $z:literal) => {
                obj.positions.push(Vec3 {
                    x: $x,
                    y: $y * c - $z * s,
                    z: $y * s + $z * c,
                });
            };
        }
        make_pos!( -1.0  0.0  1.0 );
        make_pos!(  1.0  0.0  1.0 );
        make_pos!( -1.0  0.0 -1.0 );
        make_pos!(  1.0  0.0 -1.0 );
        obj.objects.push(mol::obj::Object {
            name: "Plane".into(),
            groups: vec![mol::obj::Group {
                name: "default".into(),
                faces: vec![mol::obj::Face::Quad([
                    [1, 1, 1],
                    [2, 2, 1],
                    [4, 3, 1],
                    [3, 4, 1],
                ])],
                material: None,
            }],
        });

        obj
    }

    fn xz_plane_angle_z(z_degrees: f32) -> mol::obj::OBJ {
        let mut obj = mol::obj::OBJ::new();
        let (s, c) = z_degrees.sin_cos();
        macro_rules! make_pos {
            ($x:literal $y:literal $z:literal) => {
                obj.positions.push(Vec3 {
                    x: $x * c - $y * s,
                    y: $x * s + $y * s,
                    z: $z,
                });
            };
        }
        make_pos!( -1.0  0.0  1.0 );
        make_pos!(  1.0  0.0  1.0 );
        make_pos!( -1.0  0.0 -1.0 );
        make_pos!(  1.0  0.0 -1.0 );
        obj.objects.push(mol::obj::Object {
            name: "Plane".into(),
            groups: vec![mol::obj::Group {
                name: "default".into(),
                faces: vec![mol::obj::Face::Quad([
                    [1, 1, 1],
                    [2, 2, 1],
                    [4, 3, 1],
                    [3, 4, 1],
                ])],
                material: None,
            }],
        });

        obj
    }

    #[test]
    fn test_plane_xz_angle_x() {
        for degrees in 0..360 {
            let obj = xz_plane_angle_x((degrees as f32).to_radians());
            test_plane_obj(&obj, degrees);
        }
    }

    #[test]
    fn test_plane_xz_angle_z() {
        for degrees in 0..360 {
            let obj = xz_plane_angle_z((degrees as f32).to_radians());
            test_plane_obj(&obj, degrees);
        }
    }

    fn test_plane_obj(obj: &mol::obj::OBJ, degrees: i32) {
        let vertices = match &obj.objects[0].groups[0].faces[0] {
            mol::obj::Face::Tri(vertices) => &vertices[..],
            mol::obj::Face::Quad(vertices) => &vertices[..],
            mol::obj::Face::NGon(vertices) => &vertices[..],
        };

        let normal = earclipping::fit_plane_normal(&obj, vertices);

        let tf = Vec3::unit_align(&normal, &Vec3::UNIT_Y);

        let positions = vertices
            .iter()
            .map(|[v, ..]| obj.positions[*v - 1])
            .map(|pos| &tf * pos)
            .collect::<Vec<Vec3>>();

        macro_rules! assert_approx_pos {
            ($i:expr, ($x:literal, $y:literal, $z:literal) ) => {
                let Vec3 { x, y, z } = positions[$i];
                if (x - $x).abs() > 0.001 || (y - $y).abs() > 0.001 || (z - $z).abs() > 0.001 {
                    panic!("angle = {} deg:\n    normal: ({}, {}, {})\n    ({}, {}, {}) !≈ ({}, {}, {})", degrees, normal.x, normal.y, normal.z, x, y, z, $x, $y, $z);
                }
            };
        }
        assert_approx_pos!(0, (-1.0, 0.0, 1.0));
        assert_approx_pos!(1, (1.0, 0.0, 1.0));
        assert_approx_pos!(2, (1.0, 0.0, -1.0));
        assert_approx_pos!(3, (-1.0, 0.0, -1.0));
    }
}
