pub mod wgpu_backend;
pub mod cg_backend;

use std::marker::PhantomData;

use crate::data::{ViewResolution, PictureResolution, Picture, TessellatedPicture};
use crate::data::{Content, TessellatedContent};
use crate::data::draw_cmds::{DrawOp, FillOp, FillStrokeOp, StrokeOp};
use crate::frontend::DrawableObject;
use crate::frontend::SceneTessellator;

pub struct CanvasRenderer<const N: usize> {
    // pub gpu_handle: GpuHandle,
    // pub layers: [Option<()>; N],
}

pub trait CanvasRendererLayer {
    
}

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// INTERNAL
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Unchanged,
    Changed,
}

impl UpdateStatus {
    pub fn unchanged(&self) -> bool {*self == UpdateStatus::Unchanged}
    pub fn changed(&self) -> bool {*self == UpdateStatus::Changed}
}


