
use std::borrow::Cow;

use serde::{Serializer, Deserializer, Serialize, Deserialize};

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// GEOMETRY PRIMITIVES
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// HEAP ALLOCATED GEOMETRY TYPES
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVec {
    pub(super) points: Vec<Point>,
}

#[derive(Debug, Clone)]
pub struct PointVecRef<'a> {
    pub(super) points: &'a [Point],
}


