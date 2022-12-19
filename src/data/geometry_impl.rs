// use std::borrow::Cow;
// use serde::{Serializer, Deserializer, Serialize, Deserialize};
// use itertools::Itertools;
// use geo::{ConcaveHull, ConvexHull, Scale, BoundingRect, Intersects, Contains, EuclideanLength, Within};
// use parry2d::query::PointQuery;
// // use super::{graphics::*, geometry::*, drawing::edit_tool, drawing::stroke_style};


// //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// // GEOMETRY PRIMITIVES
// //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

// impl Point {
//     pub fn new(x: f32, y: f32) -> Self {
//         Point { x, y }
//     }
//     pub fn x(&self) -> f32 {self.x}
//     pub fn y(&self) -> f32 {self.y}
//     pub fn into_geo_coordinate(self) -> geo::Coord {
//         geo::Coord {x: self.x as f64, y: self.y as f64}
//     }
//     pub fn into_point2(self) -> parry2d::na::Point2<f32> {
//         parry2d::na::Point2::new(self.x, self.y)
//     }
//     // pub fn into_skia_point(self) -> skia_safe::Point {
//     //     skia_safe::Point::new(self.x, self.y)
//     // }
//     pub fn is_finite(self) -> bool {
//         self.x.is_finite() && self.y.is_finite()
//     }
//     pub fn total_cmp(self, other: Self) -> std::cmp::Ordering {
//         let x = self.x.total_cmp(&other.x);
//         let y = self.y.total_cmp(&other.y);
//         x.then(y)
//     }
//     pub fn apply_linear_scale(&self, scale: LinearScale) -> Self {
//         Point { x: scale.map(self.x), y: scale.map(self.y)}
//     }
//     pub fn scale_x(&self, scale: LinearScale) -> Self {
//         Point { x: scale.scale(self.x), y: self.y}
//     }
//     pub fn scale_y(&self, scale: LinearScale) -> Self {
//         Point { x: self.x, y: scale.scale(self.y)}
//     }
// }

// impl From<geo::Coord> for Point {
//     fn from(a: geo::Coord) -> Self {Point{x: a.x as f32, y: a.y as f32}}
// }
// impl From<parry2d::na::Point2<f32>> for Point {
//     fn from(a: parry2d::na::Point2<f32>) -> Self {Point{x: a.x, y: a.y}}
// }
// // impl From<skia_safe::Point> for Point {
// //     fn from(a: skia_safe::Point) -> Self {Point{x: a.x, y: a.y}}
// // }
// impl From<(f32, f32)> for Point {
//     fn from((x, y): (f32, f32)) -> Self {Point{x, y}}
// }
// impl From<[f32; 2]> for Point {
//     fn from([x, y]: [f32; 2]) -> Self {Point{x, y}}
// }
// impl From<(f64, f64)> for Point {
//     fn from((x, y): (f64, f64)) -> Self {Point{x: x as f32, y: y as f32}}
// }
// impl From<SamplePoint> for Point {
//     fn from(sample: SamplePoint) -> Self {sample.point}
// }
// impl std::ops::Mul for Point {
//     type Output = Point;
//     fn mul(self, rhs: Self) -> Self::Output {Point {x: self.x * rhs.x, y: self.y * rhs.y}}
// }
// impl std::ops::Mul<f32> for Point {
//     type Output = Point;
//     fn mul(self, constant: f32) -> Self::Output {Point {x: self.x * constant, y: self.y * constant}}
// }

// impl Rect {
//     pub fn width(&self) -> f32 {
//         self.max.x - self.min.x
//     }
//     pub fn height(&self) -> f32 {
//         self.max.y - self.min.y
//     }
//     // pub fn as_skia_rect(&self) -> skia_safe::Rect {
//     //     let width = self.width();
//     //     let height = self.height();
//     //     let min_x = self.min.x;
//     //     let min_y = self.min.y;
//     //     let top_left = skia_safe::Point::new(min_x as f32, min_y as f32);
//     //     let size = skia_safe::Size::new(width as f32, height as f32);
//     //     skia_safe::Rect::from_point_and_size(
//     //         top_left,
//     //         size
//     //     )
//     // }
//     pub fn contains_point(&self, point: impl Into<Point>) -> bool {
//         let min = self.min.into_point2();
//         let max = self.max.into_point2();
//         let point = point.into().into_point2();
//         min <= point && point <= max
//     }
//     pub fn intersects(&self, other: Rect) -> bool {
//         let min = self.min.into_point2().coords.sup(&other.min.into_point2().coords);
//         let max = self.max.into_point2().coords.inf(&other.max.into_point2().coords);
//         if min.x > max.x || min.y > max.y {
//             return false;
//         }
//         true
//     }
// }

// impl FrameSize {
//     pub fn clamped_width(&self) -> u32 {
//         if self.width > 0.0 { self.width as u32 } else { 100 }
//     }
//     pub fn clamped_height(&self) -> u32 {
//         if self.height > 0.0 { self.height as u32 } else { 100 }
//     }
// }

// //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// // HEAP ALLOCATED GEOMETRY TYPES
// //―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

// impl PointVec {
//     pub fn from_normal_vec(points: Vec<Point>) -> Self {
//         PointVec {points}
//     }
//     pub fn from_iter<T: Into<Point>>(points: impl IntoIterator<Item=T>) -> Self {
//         PointVec { points: points.into_iter().map(|x| x.into()).collect_vec() }
//     }
//     pub fn into_ref<'a>(&'a self) -> PointVecRef<'a> {
//         PointVecRef {points: &self.points}
//     }
//     pub fn push_points(&mut self, other: PointVec) {
//         self.points.extend(other.points);
//     }
//     pub fn from_nested_iter(xs: impl IntoIterator<Item=PointVec>) -> Self {
//         let points = xs
//             .into_iter()
//             .flat_map(|x| x.points)
//             .collect_vec();
//         PointVec{points}
//     }
//     pub fn is_empty(&self) -> bool {
//         self.points.is_empty()
//     }
//     pub fn map_mut(&mut self, f: impl Fn(&mut Point)) {
//         self.points
//             .iter_mut()
//             .for_each(f)
//     }
// }

// pub trait PointVecOps {
//     fn points(&self) -> &[Point];
//     fn multiply_by(&self, mul: impl Into<f32>) -> PointVec {
//         let mul = mul.into();
//         let points = self.points().into_iter().map(|a| *a * mul).collect_vec();
//         PointVec{points}
//     }
//     fn apply_linear_scale(&self, scale: LinearScale) -> PointVec {
//         let points = self.points()
//             .into_iter()
//             .map(|a| a.apply_linear_scale(scale))
//             .collect_vec();
//         PointVec{points}
//     }
//     fn into_owned(&self) -> PointVec {
//         PointVec {points: self.points().to_vec()}
//     }
//     fn into_geo_coordinates(&self) -> Vec<geo::Coord> {
//         self.points().into_iter().copied().map(Point::into_geo_coordinate).collect_vec()
//     }
//     fn into_parry2d_points(&self) -> Vec<parry2d::na::Point2<f32>> {
//         self.points().into_iter().copied().map(Point::into_point2).collect_vec()
//     }
//     // fn into_skia_points(&self) -> Vec<skia_safe::Point> {
//     //     self.points().into_iter().copied().map(Point::into_skia_point).collect_vec()
//     // }
//     fn into_parry2d_polyline(&self) -> parry2d::shape::Polyline {
//         parry2d::shape::Polyline::new(self.into_parry2d_points(), None)
//     }
//     fn into_geo_line_string(&self) -> geo::LineString {
//         geo::LineString::new(self.into_geo_coordinates())
//     }
//     fn convex_hull(&self) -> geo::Polygon {
//         self.into_geo_line_string().convex_hull()
//     }
//     fn convex_hull_exterior(&self) -> PointVec {
//         let points = self
//             .into_geo_line_string();
//         let points = points
//             .convex_hull();
//         let points = points
//             .exterior()
//             .points()
//             .map(|a| Point{x: a.x() as f32, y: a.y() as f32});
//         PointVec::from_iter(points)
//     }
//     // fn into_sk_polygon(&self) -> skia_safe::Path {
//     //     let points = self.into_skia_points();
//     //     skia_safe::Path::polygon(&points, true, None, None)
//     // }
//     fn min(&self) -> Option<Point> {
//         self.points()
//             .iter()
//             .copied()
//             .filter(|a| a.is_finite())
//             .min_by(|a, b| a.total_cmp(*b))
//     }
//     fn max(&self) -> Option<Point> {
//         self.points()
//             .into_iter()
//             .copied()
//             .filter(|a| a.is_finite())
//             .max_by(|a, b| a.total_cmp(*b))
//     }
//     fn min_x(&self) -> Option<f32> {
//         self.points()
//             .into_iter()
//             .map(Point::x)
//             .filter(|a| a.is_finite())
//             .min_by(|a, b| a.total_cmp(&b))
//     }
//     fn min_y(&self) -> Option<f32> {
//         self.points()
//             .into_iter()
//             .map(Point::y)
//             .filter(|a| a.is_finite())
//             .min_by(|a, b| a.total_cmp(&b))
//     }
//     fn max_x(&self) -> Option<f32> {
//         self.points()
//             .into_iter()
//             .map(Point::x)
//             .filter(|a| a.is_finite())
//             .max_by(|a, b| a.total_cmp(&b))
//     }
//     fn max_y(&self) -> Option<f32> {
//         self.points()
//             .into_iter()
//             .map(Point::y)
//             .filter(|a| a.is_finite())
//             .max_by(|a, b| a.total_cmp(&b))
//     }
//     fn center_point(&self) -> Point {
//         let bounds = self.convex_hull_exterior().into_parry2d_polyline();
//         let bounds = bounds.local_aabb();
//         bounds.center().to_owned().into()
//     }
//     fn aabb_any_point_overlap<T: PointVecOps>(&self, other: &T) -> bool {
//         let a = self.into_parry2d_polyline();
//         let a = a.local_aabb();
//         other
//             .into_parry2d_points()
//             .into_iter()
//             .any(|x| {
//                 a.contains_local_point(&x)
//             })
//     }
//     fn aabb_contains_point(&self, other: Point) -> bool  {
//         let bounds = self.convex_hull_exterior().into_parry2d_polyline();
//         let bounds = bounds.local_aabb();
//         bounds.contains_local_point(&other.into_point2())
//     }
//     fn intersects<T: PointVecOps>(
//         &self,
//         other: &T,
//     ) -> bool {
//         use geo::algorithm::closest_point::ClosestPoint;
//         use geo::EuclideanDistance;
//         let a = self.convex_hull();
//         let b = other.convex_hull();
//         let a = a.exterior();
//         let b = b.exterior();
//         a.intersects(b)
//     }
//     fn into_rect(&self) -> Option<Rect> {
//         let min = self.min()?;
//         let max = self.max()?;
//         Rect {min, max}.into()
//     }
// }

// impl PointVecOps for PointVec {
//     fn points(&self) -> &[Point] {&self.points}
// }
// impl PointVecOps for PointVecRef<'_> {
//     fn points(&self) -> &[Point] {self.points}
// }
