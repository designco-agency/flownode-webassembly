//! GPU acceleration scaffolding (disabled for WASM compatibility)
//!
//! WebGPU support will be added later when threading issues are resolved.
//! For now, all image processing is done on CPU.

use crate::image_data::ImageData;

/// Placeholder for GPU context - not used yet
pub struct GpuContext;

/// CPU fallback for brightness/contrast
pub fn brightness_contrast(input: &ImageData, brightness: f32, contrast: f32) -> ImageData {
    let mut pixels = (*input.pixels).clone();
    let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));
    
    for chunk in pixels.chunks_mut(4) {
        for i in 0..3 {
            let val = chunk[i] as f32;
            let val = val + brightness;
            let val = factor * (val - 128.0) + 128.0;
            chunk[i] = val.clamp(0.0, 255.0) as u8;
        }
    }
    
    ImageData::new(pixels, input.width, input.height)
}
