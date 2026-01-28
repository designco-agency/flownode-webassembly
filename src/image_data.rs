//! Image data handling for the node editor

use std::sync::Arc;

/// Raw image data that can be shared between nodes
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw RGBA pixel data
    pub pixels: Arc<Vec<u8>>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl ImageData {
    /// Create new image data from raw RGBA pixels
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        debug_assert_eq!(pixels.len(), (width * height * 4) as usize);
        Self {
            pixels: Arc::new(pixels),
            width,
            height,
        }
    }
    
    /// Create a solid color image
    pub fn solid(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixels: Vec<u8> = (0..width * height)
            .flat_map(|_| color)
            .collect();
        Self::new(pixels, width, height)
    }
    
    /// Create a checkerboard pattern (for transparency visualization)
    pub fn checkerboard(width: u32, height: u32, size: u32) -> Self {
        let pixels: Vec<u8> = (0..height)
            .flat_map(|y| {
                (0..width).flat_map(move |x| {
                    let is_light = ((x / size) + (y / size)) % 2 == 0;
                    if is_light {
                        [200u8, 200, 200, 255]
                    } else {
                        [150u8, 150, 150, 255]
                    }
                })
            })
            .collect();
        Self::new(pixels, width, height)
    }
    
    /// Get pixel at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }
    
    /// Total number of pixels
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }
    
    /// Size in bytes
    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }
}

/// Decode image from bytes (PNG, JPEG, etc.)
pub fn decode_image(bytes: &[u8]) -> Result<ImageData, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.into_raw();
    
    Ok(ImageData::new(pixels, width, height))
}

/// Encode image to PNG bytes
pub fn encode_png(data: &ImageData) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    use std::io::Cursor;
    
    let mut buffer = Cursor::new(Vec::new());
    
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    encoder.write_image(
        &data.pixels,
        data.width,
        data.height,
        image::ExtendedColorType::Rgba8,
    ).map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    Ok(buffer.into_inner())
}

/// Convert ImageData to egui ColorImage for display
pub fn to_color_image(data: &ImageData) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [data.width as usize, data.height as usize],
        &data.pixels,
    )
}

/// Texture handle for caching in egui
#[derive(Clone)]
pub struct TextureHandle {
    pub handle: egui::TextureHandle,
    pub size: [u32; 2],
}

impl TextureHandle {
    /// Create a new texture from image data
    pub fn from_image_data(ctx: &egui::Context, name: &str, data: &ImageData) -> Self {
        let color_image = to_color_image(data);
        let handle = ctx.load_texture(
            name,
            color_image,
            egui::TextureOptions::LINEAR,
        );
        Self {
            handle,
            size: [data.width, data.height],
        }
    }
    
    /// Get the texture ID for rendering
    pub fn id(&self) -> egui::TextureId {
        self.handle.id()
    }
}
