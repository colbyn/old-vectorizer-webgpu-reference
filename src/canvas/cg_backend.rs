use core_graphics::sys::CGContext;
// use core_graphics::context::


pub struct CoreGraphicsBackend<const N: usize> {
    pub layers: [Option<CanvasLayer>; N],
}

pub struct CanvasLayer {
    // context: 
}
