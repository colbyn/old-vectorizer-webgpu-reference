use crate::canvas::{CanvasRenderer, UpdateStatus};
use super::CanvasLayer;
use super::gpu_target::{GpuBackend, GpuHandle};
use crate::data::{Content, TessellatedContent};
use crate::data::gpu_types;
use crate::data::collections::CowCollection;
use wgpu::util::DeviceExt;

// impl Layer {
//     pub fn content_ref(&self) -> Content {
//         Content {
//             // items: &self.tessellated_content.picture.items[..],
//             items: unimplemented!(),
//             picture_resolution: self.tessellated_content.picture.picture_resolution,
//         }
//     }
// }
