pub mod gpu_types;
pub mod geometry;
pub mod geometry_impl;
pub mod collections;
pub mod picture;
pub mod draw_cmds;
pub mod drawable;

use std::hash::Hash;

use lyon::math::Point;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

pub use picture::{Picture, TesselationSettings, TessellatedPicture};
pub use picture::{Content, TessellatedContent};

pub struct VertexConstructor {
    pub prim_id: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct SceneGlobals {
    pub wireframe: bool,
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// BASIC DATA TYPES
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


#[derive(Debug, Clone, Copy, Default, Hash, PartialEq)]
pub struct Resolution<T> {
    width: T,
    height: T,
}

impl<T> Resolution<T> {
    pub fn new(width: T, height: T) -> Resolution<T> { Resolution { width, height } }
}

impl Resolution<u32> {
    pub fn width(&self) -> u32 {self.width * 2}
    pub fn height(&self) -> u32 {self.height * 2}
}
impl Resolution<f32> {
    pub fn width(&self) -> f32 {self.width}
    pub fn height(&self) -> f32 {self.height}
}

#[derive(Debug, Clone, Default)]
pub struct ViewInfo {
    pub view_resolution: Resolution<u32>
}


#[derive(Debug, Clone)]
pub struct View {
    pub view_resolution: Resolution<u32>,
    pub picture: Picture,
}

pub type HashValue = u64;

impl<T: Hash> Resolution<T> {
    pub fn hash_value(&self) -> HashValue {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn resolution_changed(&self, other: &Resolution<T>) -> bool {
        self.hash_value() != other.hash_value()
    }
}

impl View {
    pub fn resolution_changed(&self, other: &View) -> bool {
        self.view_resolution.resolution_changed(&other.view_resolution)
    }
}


pub type ViewResolution = Resolution<u32>;
pub type PictureResolution = Resolution<f32>;


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// BASIC DATA TYPES
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone, Copy)]
pub struct LinearScale {
    pub domain: (f32, f32),
    pub range: (f32, f32),
}

impl LinearScale {
    pub fn scale(&self, input: f32) -> f32 {
        let min_domain = self.domain.0;
        let max_domain = self.domain.1;
        let min_range = self.range.0;
        let max_range = self.range.1;
        return (max_range - min_range) * (input - min_domain) / (max_domain - min_domain) + min_range
    }
    pub fn map(&self, input: f32) -> f32 {
        self.scale(input)
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// COLOR
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RGBA<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: f32,
}


impl RGBA<f32> {
    pub fn from_u8(RGBA { red, green, blue, alpha }: RGBA<u8>) -> RGBA<f32> {
        let scale = LinearScale {
            domain: (0.0, u8::MAX as f32),
            range: (0.0, 1.0),
        };
        let red = scale.scale(red as f32);
        let green = scale.scale(green as f32);
        let blue = scale.scale(blue as f32);
        // let alpha = scale.scale(alpha as f32);
        RGBA { red, green, blue, alpha }
    }
}

impl RGBA<u8> {
    pub fn to_f32(self) -> RGBA<f32> {
        RGBA::<f32>::from_u8(self)
    }
    pub const fn new(red: u8, green: u8, blue: u8, alpha: f32) -> RGBA<u8> {
        RGBA { red, green, blue, alpha }
    }
    pub const fn new_(red: u8, green: u8, blue: u8) -> RGBA<u8> {
        RGBA { red, green, blue, alpha: 1.0 }
    }
    pub const fn with_alpha(self, alpha: f32) -> RGBA<u8> {
        let RGBA { red, green, blue, .. } = self;
        RGBA { red, green, blue, alpha }
    }
}


impl RGBA<u8> {
    pub const WHITE: RGBA<u8> = RGBA::new_(255, 255, 255);
    pub const BLACK: RGBA<u8> = RGBA::new_(0, 0, 0);
    pub const GREY: RGBA<u8> = RGBA::new_(128,128,128);
    pub const RED: RGBA<u8> = RGBA::new_(255, 0, 0);
    pub const GREEN: RGBA<u8> = RGBA::new_(0, 255, 0);
    pub const BLUE: RGBA<u8> = RGBA::new_(0, 0, 255);
    pub const CYAN: RGBA<u8> = RGBA::new_(0, 255, 255);
    pub const PURPLE: RGBA<u8> = RGBA::new_(128, 0, 128);
    pub const PINK: RGBA<u8> = RGBA::new_(255, 192, 203);
}

// impl RGBA {
//     pub fn from_u8(red, green, blue, alpha) {

//     }
// }
