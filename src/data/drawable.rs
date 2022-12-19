use super::geometry::{Point, PointVec, PointVecRef};
// use crate::data::g

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAWABLE INTERFACE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

// pub trait Drawable {
//     fn draw(&self) -> DrawOp;
// }

// pub trait DrawableScene {
//     type Cold: Drawable;
//     type Hot: Drawable;
//     fn cold_layer(&self) -> &[Self::Cold];
//     fn hot_layer(&self) -> &[Self::Hot];
// }

// pub enum Drawable {
//     Path(PointVec),
// }

pub enum DrawCmd {
    Path(PointVec, PathOptions)
}

pub struct PathOptions {

}




//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAWABLE INTERFACE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


