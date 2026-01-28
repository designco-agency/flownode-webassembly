//! GPU compute shader execution via wgpu
//! 
//! This module handles image processing through WebGPU compute shaders.

use crate::image_data::ImageData;

/// GPU processing context
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    // Cached pipelines
    brightness_contrast_pipeline: Option<wgpu::ComputePipeline>,
    hue_saturation_pipeline: Option<wgpu::ComputePipeline>,
    blur_pipeline: Option<wgpu::ComputePipeline>,
}

/// Parameters for brightness/contrast adjustment
#[derive(Debug, Clone, Copy)]
pub struct BrightnessContrastParams {
    pub brightness: f32, // -1.0 to 1.0
    pub contrast: f32,   // -1.0 to infinity
}

/// Parameters for hue/saturation adjustment
#[derive(Debug, Clone, Copy)]
pub struct HueSaturationParams {
    pub hue_shift: f32,  // -1.0 to 1.0 (full rotation)
    pub saturation: f32, // 0.0 to infinity
    pub lightness: f32,  // -1.0 to 1.0
}

/// Parameters for blur
#[derive(Debug, Clone, Copy)]
pub struct BlurParams {
    pub radius: f32,
    pub sigma: f32,
}

impl GpuContext {
    /// Create a new GPU context (async because wgpu init is async)
    pub async fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find GPU adapter")?;
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("FlowNode GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;
        
        Ok(Self {
            device,
            queue,
            brightness_contrast_pipeline: None,
            hue_saturation_pipeline: None,
            blur_pipeline: None,
        })
    }
    
    /// Check if GPU is available
    pub fn is_available(&self) -> bool {
        true // If we got here, we have a device
    }
    
    /// Apply brightness/contrast to an image
    pub fn brightness_contrast(
        &mut self,
        input: &ImageData,
        params: BrightnessContrastParams,
    ) -> Result<ImageData, String> {
        // For now, do CPU fallback - GPU implementation TODO
        let mut output = input.pixels.as_ref().clone();
        
        for chunk in output.chunks_exact_mut(4) {
            let r = chunk[0] as f32 / 255.0;
            let g = chunk[1] as f32 / 255.0;
            let b = chunk[2] as f32 / 255.0;
            
            // Apply brightness (additive)
            let r = r + params.brightness;
            let g = g + params.brightness;
            let b = b + params.brightness;
            
            // Apply contrast (around 0.5 midpoint)
            let factor = (1.0 + params.contrast).max(0.0);
            let r = ((r - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            let g = ((g - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            let b = ((b - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            
            chunk[0] = (r * 255.0) as u8;
            chunk[1] = (g * 255.0) as u8;
            chunk[2] = (b * 255.0) as u8;
        }
        
        Ok(ImageData::new(output, input.width, input.height))
    }
    
    /// Apply hue/saturation to an image
    pub fn hue_saturation(
        &mut self,
        input: &ImageData,
        params: HueSaturationParams,
    ) -> Result<ImageData, String> {
        // CPU fallback
        let mut output = input.pixels.as_ref().clone();
        
        for chunk in output.chunks_exact_mut(4) {
            let r = chunk[0] as f32 / 255.0;
            let g = chunk[1] as f32 / 255.0;
            let b = chunk[2] as f32 / 255.0;
            
            // RGB to HSL
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let l = (max + min) / 2.0;
            
            let (h, s) = if (max - min).abs() < 0.0001 {
                (0.0, 0.0)
            } else {
                let d = max - min;
                let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
                let h = if (max - r).abs() < 0.0001 {
                    (g - b) / d + if g < b { 6.0 } else { 0.0 }
                } else if (max - g).abs() < 0.0001 {
                    (b - r) / d + 2.0
                } else {
                    (r - g) / d + 4.0
                };
                (h / 6.0, s)
            };
            
            // Apply adjustments
            let h = (h + params.hue_shift).fract();
            let h = if h < 0.0 { h + 1.0 } else { h };
            let s = (s * params.saturation).clamp(0.0, 1.0);
            let l = (l + params.lightness).clamp(0.0, 1.0);
            
            // HSL to RGB
            let (r, g, b) = if s < 0.0001 {
                (l, l, l)
            } else {
                let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
                let p = 2.0 * l - q;
                let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
                    if t < 0.0 { t += 1.0; }
                    if t > 1.0 { t -= 1.0; }
                    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
                    if t < 0.5 { return q; }
                    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
                    p
                };
                (
                    hue_to_rgb(p, q, h + 1.0 / 3.0),
                    hue_to_rgb(p, q, h),
                    hue_to_rgb(p, q, h - 1.0 / 3.0),
                )
            };
            
            chunk[0] = (r * 255.0) as u8;
            chunk[1] = (g * 255.0) as u8;
            chunk[2] = (b * 255.0) as u8;
        }
        
        Ok(ImageData::new(output, input.width, input.height))
    }
}

/// CPU-only processing fallback (for when GPU isn't available)
pub mod cpu {
    use super::*;
    
    pub fn brightness_contrast(input: &ImageData, params: BrightnessContrastParams) -> ImageData {
        let mut output = input.pixels.as_ref().clone();
        
        for chunk in output.chunks_exact_mut(4) {
            let r = chunk[0] as f32 / 255.0;
            let g = chunk[1] as f32 / 255.0;
            let b = chunk[2] as f32 / 255.0;
            
            let r = (r + params.brightness).clamp(0.0, 1.0);
            let g = (g + params.brightness).clamp(0.0, 1.0);
            let b = (b + params.brightness).clamp(0.0, 1.0);
            
            let factor = (1.0 + params.contrast).max(0.0);
            let r = ((r - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            let g = ((g - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            let b = ((b - 0.5) * factor + 0.5).clamp(0.0, 1.0);
            
            chunk[0] = (r * 255.0) as u8;
            chunk[1] = (g * 255.0) as u8;
            chunk[2] = (b * 255.0) as u8;
        }
        
        ImageData::new(output, input.width, input.height)
    }
}
