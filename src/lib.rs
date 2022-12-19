#![allow(unused)]

pub mod data;
pub mod canvas;
pub mod frontend;

use std::borrow::Cow;
use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};
use wgpu::{include_wgsl, SurfaceConfiguration};
use futures::executor::block_on;
use wgpu::util::DeviceExt;
use std::f64::NAN;

pub use data::View;
pub use data::{ViewResolution, PictureResolution};
pub use data::{Picture, TesselationSettings, TessellatedPicture};

