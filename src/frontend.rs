use crate::data::draw_cmds::{DrawOp, FillOp, FillStrokeOp, StrokeOp};
use crate::{ViewResolution, PictureResolution};
use crate::data::{gpu_types, RGBA};
use crate::data::collections::CowCollection;
use crate::data::geometry::{Point, PointVec, PointVecRef};
use std::borrow::Cow;
use lyon::tessellation::geometry_builder::VertexBuffers;

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAWABLE INTERFACE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub trait DrawableObject {
    fn draw(&self) -> DrawOp;
}

impl<'a, T: DrawableObject> DrawableObject for &'a T {
    fn draw(&self) -> DrawOp {(*self).draw()}
}

impl DrawableObject for DrawOp {
    fn draw(&self) -> crate::data::draw_cmds::DrawOp { self.clone() }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAWABLE INTERFACE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――




//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// SCENE-TESSELLATOR
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub struct SceneTessellator {
    pub mesh: VertexBuffers<gpu_types::GpuVertex, u32>,
    pub primitives: Vec<gpu_types::GpuPrimitive>,
    pub fill_tessellator: lyon::tessellation::FillTessellator,
    pub stroke_tessellator: lyon::tessellation::StrokeTessellator,
    pub picture_resolution: PictureResolution,
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// SCENE-TESSELLATOR API
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

impl SceneTessellator {
    pub fn new(picture_resolution: PictureResolution) -> Self {
        let mesh: VertexBuffers<gpu_types::GpuVertex, u32> = VertexBuffers::new();
        let primitives: Vec<gpu_types::GpuPrimitive> = Vec::new();
        let fill_tessellator: lyon::tessellation::FillTessellator = lyon::tessellation::FillTessellator::new();
        let stroke_tessellator: lyon::tessellation::StrokeTessellator = lyon::tessellation::StrokeTessellator::new();
        SceneTessellator { mesh, primitives, fill_tessellator, stroke_tessellator, picture_resolution }
    }
    pub fn append_draw_op(&mut self, object: impl Into<DrawOp>) {
        use lyon::tessellation::geometry_builder::*;
        match object.into() {
            DrawOp::Fill(FillOp { path, fill_color, fill_settings }) => {
                self.primitives.push({
                    gpu_types::GpuPrimitive::from_u8_rgba(fill_color)
                });
                let fill_color_ix = self.primitives.len() as u32 - 1;
                let _: () = self.fill_tessellator.tessellate_path(
                        &path,
                        &fill_settings,
                        &mut BuffersBuilder::new(
                            &mut self.mesh,
                            crate::data::VertexConstructor {
                                prim_id: fill_color_ix,
                            },
                        ),
                    )
                    .expect("Error during tesselation!");
            }
            DrawOp::Stroke(StrokeOp { path, stroke_color, stroke_settings }) => {
                self.primitives.push({
                    gpu_types::GpuPrimitive::from_u8_rgba(stroke_color)
                });
                let stroke_color_ix = self.primitives.len() as u32 - 1;
                let _: () = self.stroke_tessellator.tessellate_path(
                        &path,
                        &stroke_settings,
                        &mut BuffersBuilder::new(
                            &mut self.mesh,
                            crate::data::VertexConstructor {
                                prim_id: stroke_color_ix,
                            },
                        ),
                    )
                    .expect("Error during tesselation!");
            }
            DrawOp::FillStroke(FillStrokeOp { path, fill_color, stroke_color, fill_settings, stroke_settings }) => {
                self.primitives.push({
                    gpu_types::GpuPrimitive::from_u8_rgba(fill_color)
                });
                let fill_color_ix = self.primitives.len() as u32 - 1;
                self.primitives.push({
                    gpu_types::GpuPrimitive::from_u8_rgba(stroke_color)
                });
                let stroke_color_ix = self.primitives.len() as u32 - 1;
                let _: () = self.fill_tessellator.tessellate_path(
                        &path,
                        &fill_settings,
                        &mut BuffersBuilder::new(
                            &mut self.mesh,
                            crate::data::VertexConstructor {
                                prim_id: fill_color_ix,
                            },
                        ),
                    )
                    .expect("Error during tesselation!");
                let _: () = self.stroke_tessellator.tessellate_path(
                        &path,
                        &stroke_settings,
                        &mut BuffersBuilder::new(
                            &mut self.mesh,
                            crate::data::VertexConstructor {
                                prim_id: stroke_color_ix,
                            },
                        ),
                    )
                    .expect("Error during tesselation!");
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct DynamicStroke {
    color: RGBA<u8>,
    path: PointVec,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMode {
    Fill,
    Stroke,
    FillStroke,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    Dark,
    Light,
}

// pub struct Scene {
    
// }

// impl Scene {
//     // pub fn append_path(&mut self, points: )
// }

// pub enum SceneObject {
//     DrawOp(DrawOp),
//     DynamicPath()
// }


// pub trait Vectorizable {
//     fn color(&self, color_scheme: ColorScheme) -> RGBA<u8> {
//         match color_scheme {
//             ColorScheme::Dark => RGBA::WHITE,
//             ColorScheme::Light => RGBA::BLACK,
//         }
//     }
//     fn path(&self) -> PointVecRef<'_>;
// }

// impl Vectorizable for DrawableObject {

// }

// pub trait Drawable {
//     fn draw(&self, scene_tessellator: &mut SceneTessellator);
// }



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DRAW API
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――



