use crate::data::{self, Resolution, PictureResolution};
use crate::data::collections::CowCollection;
use crate::data::draw_cmds::*;
use crate::frontend::DrawableObject;
use itertools::Itertools;
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

use super::gpu_types::GpuVertex;

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TESSELATION-SETTINGS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug)]
pub struct TesselationSettings {
    tolerance: f32,
}

impl Default for TesselationSettings {
    fn default() -> Self {
        TesselationSettings {
            tolerance: TesselationSettings::DEFAULT_TOLERANCE,
        }
    }
}

impl TesselationSettings {
    // pub const DEFAULT_TOLERANCE: f32 = lyon::tessellation::FillOptions::DEFAULT_TOLERANCE;
    pub const DEFAULT_TOLERANCE: f32 = 0.01;

    pub const fn default_fill_options() -> lyon::tessellation::FillOptions {
        lyon::tessellation::FillOptions::DEFAULT
            .with_tolerance(TesselationSettings::DEFAULT_TOLERANCE)
            .with_intersections(true)
    }
    pub const fn default_stroke_options() -> lyon::tessellation::StrokeOptions {
        lyon::tessellation::StrokeOptions::DEFAULT
            .with_tolerance(TesselationSettings::DEFAULT_TOLERANCE)
    }
}

const STROKE_WIDTH: lyon::path::AttributeIndex = 0;

//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// PICTURE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone)]
pub struct Content<T, F: IntoIterator<Item = T> = Vec<T>> {
    pub(crate) items: F,
    pub(crate) picture_resolution: Resolution<f32>,
}

pub type Picture = Content<DrawOp>;

impl Picture {
    pub fn append(&mut self, op: impl Into<DrawOp>) {
        self.items.push(op.into());
    }
}

impl<T> Content<T> {
    pub fn as_ref(&self) -> Content<&T, &[T]> {
        Content {
            items: self.items.as_slice(),
            picture_resolution: self.picture_resolution,
        }
    }
}

impl<'a, T> Content<&'a T, &'a [T]> where T: Clone {
    pub fn into_owned(&self) -> Content<T> {
        Content {
            items: self.items.to_vec(),
            picture_resolution: self.picture_resolution,
        }
    }
}

// pub trait IntoContent<T, F: IntoIterator<Item = T> = Vec<T>> {
//     fn into_content(self) -> Content<T, F>;
//     // fn changed(&self, other: );
// }

// struct Dev<T, F> {
//     xs: dyn IntoContent<T, F>,
// }

// pub trait DrawableCollection {
//     fn picture_resolution(&self) -> PictureResolution;
//     fn items_ref<T: DrawableObject>(&self) -> Option<&[T]>;
//     fn items_owned<T: DrawableObject>(&self) -> Vec<T>;
// }

// pub struct DrawableCollection<T, F: IntoIterator<Item=T> = Vec<T>> {
//     items: F
// }

// impl<F, T> DrawableCollection<T, F> where F: IntoIterator<Item=T> {
    
// }

// impl<T, F> DrawableCollection<T, F> where F: IntoIterator<Item=T> + Default {
//     fn new() -> Self {
//         DrawableCollection{items: Default::default()}
//     }
// }

// fn dev() {
//     let xs: DrawableCollection<&DrawOp, &[DrawOp]> = unimplemented!();
//     let ys: DrawableCollection<DrawOp> = unimplemented!();
//     for x in xs.items {

//     }
//     for y in ys.items.iter() {

//     }
// }

// pub struct DrawableCollection {

// }


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// PICTURE METHODS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

// impl Picture<> {
//     pub fn needs_update(&self, other: Picture) -> bool {
//         unimplemented!()
//     }
// }



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TESSELLATED-PICTURE
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

pub(crate) type MeshBuffer = VertexBuffers<data::gpu_types::GpuVertex, u32>;
pub(crate) type GpuPrimitives = Vec<data::gpu_types::GpuPrimitive>;


#[derive(Debug)]
pub struct TessellatedContent {
    pub(crate) mesh: MeshBuffer,
    pub(crate) primitives: GpuPrimitives,
    pub(crate) picture_resolution: Resolution<f32>,
    pub(crate) needs_update: bool,
}


pub type TessellatedPicture = TessellatedContent;


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// PICTURE - RUN TESSELATOR
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


impl<T> Content<T, Vec<T>> where T: DrawableObject {
    pub(crate) fn tessellate(&self) -> TessellatedContent { self.as_ref().tessellate() }
}

impl<'a, T> Content<&'a T, &'a [T]> where T: DrawableObject {
    pub(crate) fn tessellate(&self) -> TessellatedContent {
        let mut mesh: VertexBuffers<GpuVertex, u32> = VertexBuffers::new();
        let mut primitives: Vec<data::gpu_types::GpuPrimitive> = Vec::new();
        let mut fill_tessellator: FillTessellator = FillTessellator::new();
        let mut stroke_tessellator: StrokeTessellator = StrokeTessellator::new();
        for op in self.items.into_iter() {
            match op.draw() {
                DrawOp::Fill(FillOp { path, fill_color, fill_settings }) => {
                    primitives.push({
                        data::gpu_types::GpuPrimitive::from_u8_rgba(fill_color)
                    });
                    let fill_color_ix = primitives.len() as u32 - 1;
                    let _: () = fill_tessellator.tessellate_path(
                            &path,
                            &fill_settings,
                            &mut BuffersBuilder::new(
                                &mut mesh,
                                data::VertexConstructor {
                                    prim_id: fill_color_ix,
                                },
                            ),
                        )
                        .expect("Error during tesselation!");
                }
                DrawOp::Stroke(StrokeOp { path, stroke_color, stroke_settings }) => {
                    primitives.push({
                        data::gpu_types::GpuPrimitive::from_u8_rgba(stroke_color)
                    });
                    let stroke_color_ix = primitives.len() as u32 - 1;
                    let _: () = stroke_tessellator.tessellate_path(
                            &path,
                            &stroke_settings,
                            &mut BuffersBuilder::new(
                                &mut mesh,
                                data::VertexConstructor {
                                    prim_id: stroke_color_ix,
                                },
                            ),
                        )
                        .expect("Error during tesselation!");
                }
                DrawOp::FillStroke(FillStrokeOp { path, fill_color, stroke_color, fill_settings, stroke_settings }) => {
                    primitives.push({
                        data::gpu_types::GpuPrimitive::from_u8_rgba(fill_color)
                    });
                    let fill_color_ix = primitives.len() as u32 - 1;
                    primitives.push({
                        data::gpu_types::GpuPrimitive::from_u8_rgba(stroke_color)
                    });
                    let stroke_color_ix = primitives.len() as u32 - 1;
                    let _: () = fill_tessellator.tessellate_path(
                            &path,
                            &fill_settings,
                            &mut BuffersBuilder::new(
                                &mut mesh,
                                data::VertexConstructor {
                                    prim_id: fill_color_ix,
                                },
                            ),
                        )
                        .expect("Error during tesselation!");
                    let _: () = stroke_tessellator.tessellate_path(
                            &path,
                            &stroke_settings,
                            &mut BuffersBuilder::new(
                                &mut mesh,
                                data::VertexConstructor {
                                    prim_id: stroke_color_ix,
                                },
                            ),
                        )
                        .expect("Error during tesselation!");
                }
            }
        }
        TessellatedContent {
            mesh,
            primitives,
            picture_resolution: self.picture_resolution,
            needs_update: true,
        }
    }
}



//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// DEV
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


impl Picture {
    pub fn sample_picture() -> Picture {
        use lyon::path::math::point;
        let tesselation_settings = TesselationSettings::default();
        let viewport = PictureResolution { width: 1000.0, height: 1000.0 };
        let shape0 = {
            let x_offset = viewport.width * 0.2;
            let y_offset = viewport.height * 0.2;
            let mut builder = lyon::path::Path::builder();
            let center = point(
                viewport.width * 0.45,
                viewport.height * 0.3,
            );
            // builder.begin(center);
            let radii = lyon::math::Vector::from((100.0, 100.0));
            let winding = lyon::path::Winding::Positive;
            builder.add_ellipse(center, radii, lyon::math::Angle::degrees(360.0), winding);
            // builder.end(true);
            // builder.move_to(center);
            // builder.arc(center, radii, lyon::math::Angle::degrees(360.0), lyon::math::Angle::degrees(0.0));
            // builder.close();
            let path = builder.build();
            let fill = FillStrokeOp {
                path,
                fill_color: data::RGBA::BLUE.with_alpha(0.7),
                stroke_color: data::RGBA::BLACK.with_alpha(1.0),
                fill_settings: TesselationSettings::default_fill_options(),
                stroke_settings: TesselationSettings::default_stroke_options()
                    .with_line_width(5.0)
                    .with_line_cap(lyon::path::LineCap::Round),
            };
            DrawOp::FillStroke(fill)
        };
        let shape1 = {
            let x_offset = viewport.width * 0.1;
            let y_offset = viewport.height * 0.1;
            let mut builder = lyon::path::Path::builder();
            builder.begin(point(
                x_offset,
                y_offset
            ));
            builder.line_to(point(
                viewport.width - x_offset,
                y_offset
            ));
            builder.line_to(point(
                viewport.width - x_offset,
                viewport.height - y_offset
            ));
            builder.line_to(point(
                x_offset,
                viewport.height - y_offset
            ));
            builder.close();
            let path = builder.build();
            let fill_stroke = FillStrokeOp {
                path,
                fill_color: data::RGBA::CYAN.with_alpha(1.0),
                fill_settings: TesselationSettings::default_fill_options(),
                stroke_color: data::RGBA::BLACK.with_alpha(1.0),
                stroke_settings: TesselationSettings::default_stroke_options()
                    .with_line_width(10.0)
                    .with_line_cap(lyon::path::LineCap::Round),
            };
            DrawOp::FillStroke(fill_stroke)
        };
        let shape2 = {
            let x_offset = viewport.width * 0.2;
            let y_offset = viewport.height * 0.2;
            let mut builder = lyon::path::Path::builder();
            builder.begin(point(
                viewport.width / 2.0,
                y_offset
            ));
            builder.line_to(point(
                viewport.width - x_offset,
                viewport.height - y_offset
            ));
            builder.line_to(point(
                x_offset,
                viewport.height - y_offset
            ));
            builder.close();
            let path = builder.build();
            let fill = FillStrokeOp {
                path,
                fill_color: data::RGBA::GREY.with_alpha(0.9),
                fill_settings: TesselationSettings::default_fill_options(),
                stroke_color: data::RGBA::RED.with_alpha(0.9),
                stroke_settings: TesselationSettings::default_stroke_options()
                    .with_line_width(10.0)
                    .with_line_cap(lyon::path::LineCap::Round),
            };
            DrawOp::FillStroke(fill)
        };
        let shape3 = {
            let mut builder = lyon::path::Path::builder_with_attributes(1);
            builder.begin(
                point(40.0, 100.0),
                &[10.0]
            );
            builder.line_to(
                point(80.0, 20.0),
                &[5.0]
            );
            builder.cubic_bezier_to(
                point(100.0, 220.0),
                point(150.0, 50.0),
                point(250.0, 100.0),
                &[5.0]
            );
            builder.line_to(
                point(200.0, 50.0),
                &[10.0]
            );
            builder.end(false);
            let path = builder.build();

            let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();

            let stroke_settings = lyon::tessellation::StrokeOptions::tolerance(0.01)
                .with_line_cap(lyon::path::LineCap::Round)
                .with_variable_line_width(STROKE_WIDTH);
            
            let stroke = StrokeOp {
                path,
                stroke_color: data::RGBA::RED.with_alpha(0.9),
                stroke_settings
            };
            DrawOp::Stroke(stroke)
        };
        let draw = vec![
            shape1,
            shape2,
            shape0,
            shape3,
        ];
        Picture { items: draw, picture_resolution: viewport }
    }
}



impl Picture {
    pub fn from<T>(frame: PictureResolution, scene: impl IntoIterator<Item = T>) -> Picture where T: DrawableObject {
        let items = scene.into_iter().map(|x| x.draw()).collect_vec();
        Picture {picture_resolution: frame, items}
    }
}

// impl TessellatedPicture {
//     pub fn append<T>(&mut self, new_item: T) where T: DrawableObject {
        
//     }
// }


