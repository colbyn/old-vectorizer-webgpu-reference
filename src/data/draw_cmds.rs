use crate::data::{self, Resolution, PictureResolution};
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAW OPERATIONS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone)]
pub enum DrawOp {
    Stroke(StrokeOp),
    Fill(FillOp),
    FillStroke(FillStrokeOp),
}

#[derive(Debug, Clone)]
pub struct StrokeOp {
    pub path: lyon::path::Path,
    pub stroke_color: crate::data::RGBA<u8>,
    pub stroke_settings: lyon::tessellation::StrokeOptions,
}

#[derive(Debug, Clone)]
pub struct FillOp {
    pub path: lyon::path::Path,
    pub fill_color: crate::data::RGBA<u8>,
    pub fill_settings: lyon::tessellation::FillOptions,
}

#[derive(Debug, Clone)]
pub struct FillStrokeOp {
    pub path: lyon::path::Path,
    pub fill_color: crate::data::RGBA<u8>,
    pub stroke_color: crate::data::RGBA<u8>,
    pub fill_settings: lyon::tessellation::FillOptions,
    pub stroke_settings: lyon::tessellation::StrokeOptions,
}


impl From<FillStrokeOp> for DrawOp {
    fn from(op: FillStrokeOp) -> Self { DrawOp::FillStroke(op) }
}
impl From<FillOp> for DrawOp {
    fn from(op: FillOp) -> Self { DrawOp::Fill(op) }
}
impl From<StrokeOp> for DrawOp {
    fn from(op: StrokeOp) -> Self { DrawOp::Stroke(op) }
}



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAW API
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――



pub trait Drawable {
    fn draw_op(&self) -> &DrawOp;
    fn changed(&self) -> bool;
    fn drawn(&mut self);
}

pub trait DrawableCollection {
    type Item: Drawable;
    fn picture_resolution(&self) -> PictureResolution;
    fn any_changed(&self) -> bool;
    fn drawables(&self) -> &[Self::Item];
}
