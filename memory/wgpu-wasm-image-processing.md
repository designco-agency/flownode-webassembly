# WGPU for Rust WebAssembly Image Processing

A comprehensive guide to using wgpu for GPU-accelerated image processing in Rust WebAssembly applications.

---

## Table of Contents

1. [Overview: How WGPU Works in WASM Context](#1-overview-how-wgpu-works-in-wasm-context)
2. [Compute Shaders for Image Operations](#2-compute-shaders-for-image-operations)
3. [Buffer Management Between CPU and GPU](#3-buffer-management-between-cpu-and-gpu)
4. [Texture Handling and Formats](#4-texture-handling-and-formats)
5. [Performance Characteristics vs CPU-based Image Crate](#5-performance-characteristics-vs-cpu-based-image-crate)
6. [Example Code Patterns](#6-example-code-patterns)
7. [Limitations in Browser WASM Context](#7-limitations-in-browser-wasm-context)
8. [Fallback Strategies](#8-fallback-strategies)

---

## 1. Overview: How WGPU Works in WASM Context

### What is wgpu?

**wgpu** is a safe, portable graphics library for Rust based on the WebGPU API. It provides:

- **Cross-platform support**: Runs natively on Vulkan, Metal, DirectX 12, and OpenGL ES
- **Web support**: Runs in browsers via WebAssembly on WebGPU and WebGL2
- **Unified API**: Same Rust code works on all platforms

### Architecture in WASM

```
┌─────────────────────────────────────────────────────────────┐
│                     Rust Application                        │
├─────────────────────────────────────────────────────────────┤
│                        wgpu crate                           │
├─────────────────────────────────────────────────────────────┤
│  Native (Vulkan/Metal/DX12)  │  WASM (WebGPU/WebGL2)       │
├──────────────────────────────┼──────────────────────────────┤
│       GPU Drivers            │  Browser WebGPU API         │
│                              │  (via wasm-bindgen/web-sys) │
└──────────────────────────────┴──────────────────────────────┘
```

### Setting Up for WASM

**Cargo.toml Configuration:**

```toml
[lib]
crate-type = ["cdylib", "rlib"]  # cdylib for WASM, rlib for native

[dependencies]
wgpu = { version = "0.28", features = ["webgpu", "webgl"] }
pollster = "0.4"  # For blocking on async (native only)

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
console_error_panic_hook = "0.1"
console_log = "1.0"
web-sys = { version = "0.3", features = ["Window", "Document", "Element"] }
```

**Build Commands:**

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build for WebGPU (modern browsers)
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown

# Build for WebGL2 (broader compatibility)
cargo build --target wasm32-unknown-unknown --features webgl
```

### Initialization Pattern

```rust
use wgpu::Instance;

// Works on both native and WASM
pub async fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = Instance::new(&wgpu::InstanceDescriptor {
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
        .expect("Failed to find adapter");
    
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Image Processing Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        )
        .await
        .expect("Failed to create device");
    
    (device, queue)
}
```

---

## 2. Compute Shaders for Image Operations

### WGSL Shader Language

wgpu uses **WGSL** (WebGPU Shading Language) for shaders. WGSL is always supported and is the native shader language for WebGPU.

### Workgroup Concepts

```
┌────────────────────────────────────────────────────────────┐
│                    dispatchWorkgroups(4, 3, 2)             │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                      │
│  │ WG   │ │ WG   │ │ WG   │ │ WG   │   ... (4 × 3 × 2)    │
│  │(0,0,0)│(1,0,0)│(2,0,0)│(3,0,0)│                        │
│  └──────┘ └──────┘ └──────┘ └──────┘                      │
│                                                            │
│  Each Workgroup contains:                                  │
│  @workgroup_size(8, 8, 1) = 64 threads                    │
│  ┌─────────────────────┐                                  │
│  │ T T T T T T T T     │  local_invocation_id: (0-7, 0-7) │
│  │ T T T T T T T T     │  global_invocation_id: unique    │
│  │ ... (8×8 threads)   │                                  │
│  └─────────────────────┘                                  │
└────────────────────────────────────────────────────────────┘
```

**Key builtins:**
- `local_invocation_id`: Thread ID within workgroup
- `workgroup_id`: Workgroup ID within dispatch
- `global_invocation_id`: Unique ID = workgroup_id × workgroup_size + local_invocation_id

### Box Blur Shader

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

const BLUR_RADIUS: i32 = 3;

@compute @workgroup_size(8, 8, 1)
fn box_blur(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    let coords = vec2<i32>(gid.xy);
    
    // Check bounds
    if (coords.x >= i32(dims.x) || coords.y >= i32(dims.y)) {
        return;
    }
    
    var color_sum = vec4<f32>(0.0);
    var count = 0.0;
    
    // Sample surrounding pixels
    for (var dy = -BLUR_RADIUS; dy <= BLUR_RADIUS; dy++) {
        for (var dx = -BLUR_RADIUS; dx <= BLUR_RADIUS; dx++) {
            let sample_coords = coords + vec2<i32>(dx, dy);
            
            // Clamp to image bounds
            let clamped = clamp(sample_coords, vec2<i32>(0), vec2<i32>(dims) - 1);
            let pixel = textureLoad(input_texture, vec2<u32>(clamped), 0);
            color_sum += pixel;
            count += 1.0;
        }
    }
    
    textureStore(output_texture, vec2<u32>(coords), color_sum / count);
}
```

### Gaussian Blur Shader

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> direction: vec2<f32>; // (1,0) for horizontal, (0,1) for vertical

// 5-tap Gaussian weights (sigma ≈ 1.4)
const WEIGHTS: array<f32, 5> = array<f32, 5>(
    0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
);

@compute @workgroup_size(8, 8, 1)
fn gaussian_blur(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    let coords = vec2<i32>(gid.xy);
    
    if (coords.x >= i32(dims.x) || coords.y >= i32(dims.y)) {
        return;
    }
    
    var result = textureLoad(input_texture, vec2<u32>(coords), 0) * WEIGHTS[0];
    
    for (var i = 1; i < 5; i++) {
        let offset = vec2<i32>(direction * f32(i));
        let pos = clamp(coords + offset, vec2<i32>(0), vec2<i32>(dims) - 1);
        let neg = clamp(coords - offset, vec2<i32>(0), vec2<i32>(dims) - 1);
        
        result += textureLoad(input_texture, vec2<u32>(pos), 0) * WEIGHTS[i];
        result += textureLoad(input_texture, vec2<u32>(neg), 0) * WEIGHTS[i];
    }
    
    textureStore(output_texture, vec2<u32>(coords), result);
}
```

### Sharpen Shader

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> strength: f32;

// Sharpening kernel (3x3)
// [ 0, -1,  0]
// [-1,  5, -1]
// [ 0, -1,  0]

@compute @workgroup_size(8, 8, 1)
fn sharpen(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    let coords = vec2<i32>(gid.xy);
    
    if (coords.x >= i32(dims.x) || coords.y >= i32(dims.y)) {
        return;
    }
    
    let center = textureLoad(input_texture, vec2<u32>(coords), 0);
    
    // Sample neighbors (clamped)
    let top = textureLoad(input_texture, vec2<u32>(clamp(coords + vec2(0, -1), vec2(0), vec2<i32>(dims) - 1)), 0);
    let bottom = textureLoad(input_texture, vec2<u32>(clamp(coords + vec2(0, 1), vec2(0), vec2<i32>(dims) - 1)), 0);
    let left = textureLoad(input_texture, vec2<u32>(clamp(coords + vec2(-1, 0), vec2(0), vec2<i32>(dims) - 1)), 0);
    let right = textureLoad(input_texture, vec2<u32>(clamp(coords + vec2(1, 0), vec2(0), vec2<i32>(dims) - 1)), 0);
    
    // Apply kernel
    let sharpened = center * 5.0 - (top + bottom + left + right);
    
    // Blend between original and sharpened based on strength
    let result = mix(center, sharpened, strength);
    
    textureStore(output_texture, vec2<u32>(coords), clamp(result, vec4(0.0), vec4(1.0)));
}
```

### Color Adjustment Shader (Brightness, Contrast, Saturation)

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

struct ColorParams {
    brightness: f32,  // -1.0 to 1.0
    contrast: f32,    // 0.0 to 2.0 (1.0 = normal)
    saturation: f32,  // 0.0 to 2.0 (1.0 = normal)
    hue_shift: f32,   // 0.0 to 1.0 (rotation)
}

@group(0) @binding(2) var<uniform> params: ColorParams;

// sRGB luminance weights
const LUMINANCE_WEIGHTS = vec3<f32>(0.2126, 0.7152, 0.0722);

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(rgb.r, max(rgb.g, rgb.b));
    let cmin = min(rgb.r, min(rgb.g, rgb.b));
    let delta = cmax - cmin;
    
    var h = 0.0;
    if (delta > 0.0) {
        if (cmax == rgb.r) {
            h = ((rgb.g - rgb.b) / delta) % 6.0;
        } else if (cmax == rgb.g) {
            h = (rgb.b - rgb.r) / delta + 2.0;
        } else {
            h = (rgb.r - rgb.g) / delta + 4.0;
        }
        h /= 6.0;
        if (h < 0.0) { h += 1.0; }
    }
    
    let s = select(0.0, delta / cmax, cmax > 0.0);
    return vec3<f32>(h, s, cmax);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x * 6.0;
    let s = hsv.y;
    let v = hsv.z;
    
    let c = v * s;
    let x = c * (1.0 - abs(h % 2.0 - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    let hi = i32(h) % 6;
    switch (hi) {
        case 0: { rgb = vec3(c, x, 0.0); }
        case 1: { rgb = vec3(x, c, 0.0); }
        case 2: { rgb = vec3(0.0, c, x); }
        case 3: { rgb = vec3(0.0, x, c); }
        case 4: { rgb = vec3(x, 0.0, c); }
        default: { rgb = vec3(c, 0.0, x); }
    }
    return rgb + m;
}

@compute @workgroup_size(8, 8, 1)
fn color_adjust(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (gid.x >= dims.x || gid.y >= dims.y) {
        return;
    }
    
    var color = textureLoad(input_texture, gid.xy, 0);
    
    // Brightness
    color = vec4(color.rgb + params.brightness, color.a);
    
    // Contrast (around 0.5 midpoint)
    color = vec4((color.rgb - 0.5) * params.contrast + 0.5, color.a);
    
    // Saturation
    let luminance = dot(color.rgb, LUMINANCE_WEIGHTS);
    let gray = vec3(luminance);
    color = vec4(mix(gray, color.rgb, params.saturation), color.a);
    
    // Hue shift (if needed)
    if (params.hue_shift != 0.0) {
        var hsv = rgb_to_hsv(color.rgb);
        hsv.x = fract(hsv.x + params.hue_shift);
        color = vec4(hsv_to_rgb(hsv), color.a);
    }
    
    textureStore(output_texture, gid.xy, clamp(color, vec4(0.0), vec4(1.0)));
}
```

---

## 3. Buffer Management Between CPU and GPU

### Buffer Types and Usages

```rust
use wgpu::BufferUsages;

// Buffer for reading results back to CPU
let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Read Buffer"),
    size: data_size,
    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
    mapped_at_creation: false,
});

// Buffer for uploading data from CPU
let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Staging Buffer"),
    size: data_size,
    usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
    mapped_at_creation: true,  // Can write immediately
});

// GPU-only storage buffer
let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Storage Buffer"),
    size: data_size,
    usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
    mapped_at_creation: false,
});
```

### Upload Pattern (CPU → GPU)

```rust
// Method 1: queue.write_buffer (simplest)
queue.write_buffer(&gpu_buffer, 0, &cpu_data);

// Method 2: Staging buffer with mapping (more control)
let staging = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Staging"),
    contents: &cpu_data,
    usage: BufferUsages::COPY_SRC,
});

let mut encoder = device.create_command_encoder(&Default::default());
encoder.copy_buffer_to_buffer(&staging, 0, &gpu_buffer, 0, cpu_data.len() as u64);
queue.submit([encoder.finish()]);
```

### Download Pattern (GPU → CPU)

```rust
async fn read_buffer_data(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
    size: u64,
) -> Vec<u8> {
    // Create a mappable buffer
    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Read Buffer"),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    // Copy from GPU buffer to mappable buffer
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(source, 0, &read_buffer, 0, size);
    queue.submit([encoder.finish()]);
    
    // Map and read
    let buffer_slice = read_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    
    // Poll until ready (required on native)
    device.poll(wgpu::Maintain::Wait);
    rx.await.unwrap().unwrap();
    
    // Copy data
    let data = buffer_slice.get_mapped_range().to_vec();
    read_buffer.unmap();
    
    data
}
```

### Memory Layout Considerations

```
┌──────────────────────────────────────────────────────────────┐
│ WASM (Browser)                                               │
│ ┌─────────────────┐    ┌─────────────────┐                  │
│ │ WebAssembly     │    │ Browser GPU     │                  │
│ │ Linear Memory   │←──→│ Process         │                  │
│ │ (your Rust data)│ IPC│ (actual GPU     │                  │
│ │                 │    │  buffers)       │                  │
│ └─────────────────┘    └─────────────────┘                  │
│                                                              │
│ Note: All buffer mapping involves copies through IPC        │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Native                                                       │
│ ┌─────────────────┐    ┌─────────────────┐                  │
│ │ CPU Memory      │←──→│ GPU Memory      │                  │
│ │                 │    │ (may be shared  │                  │
│ │                 │    │  on integrated) │                  │
│ └─────────────────┘    └─────────────────┘                  │
│                                                              │
│ Note: Some systems have unified memory (faster transfers)   │
└──────────────────────────────────────────────────────────────┘
```

---

## 4. Texture Handling and Formats

### Common Texture Formats for Image Processing

| Format | Bytes/Pixel | Description | Use Case |
|--------|-------------|-------------|----------|
| `Rgba8Unorm` | 4 | 8-bit per channel, 0-255 → 0.0-1.0 | Standard images |
| `Rgba8UnormSrgb` | 4 | sRGB color space | Photos, UI |
| `Rgba16Float` | 8 | 16-bit float per channel | HDR, precision |
| `Rgba32Float` | 16 | 32-bit float per channel | Maximum precision |
| `R8Unorm` | 1 | Single channel | Grayscale, masks |
| `Rg8Unorm` | 2 | Two channels | Normal maps |

### Creating Textures

```rust
fn create_image_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    data: &[u8],  // RGBA8 data
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Image Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING 
             | wgpu::TextureUsages::COPY_DST
             | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),  // RGBA = 4 bytes
            rows_per_image: Some(height),
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    
    texture
}
```

### Storage Textures (for Compute Shader Output)

```rust
// Storage texture for compute shader write
let output_texture = device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Output Texture"),
    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::Rgba8Unorm,  // Must support storage
    usage: wgpu::TextureUsages::STORAGE_BINDING 
         | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
});

// Create storage view
let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
    format: Some(wgpu::TextureFormat::Rgba8Unorm),
    ..Default::default()
});
```

### Reading Texture Data Back

```rust
async fn read_texture_data(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let bytes_per_pixel = 4u32;  // RGBA8
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;
    
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Texture Read Buffer"),
        size: (padded_bytes_per_row * height) as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit([encoder.finish()]);
    
    // Map and read (removing row padding)
    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    device.poll(wgpu::Maintain::Wait);
    rx.await.unwrap().unwrap();
    
    let padded_data = buffer_slice.get_mapped_range();
    let mut result = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
    
    // Remove row padding
    for row in 0..height {
        let start = (row * padded_bytes_per_row) as usize;
        let end = start + (width * bytes_per_pixel) as usize;
        result.extend_from_slice(&padded_data[start..end]);
    }
    
    result
}
```

---

## 5. Performance Characteristics vs CPU-based Image Crate

### Performance Comparison Matrix

| Operation | CPU (image crate) | GPU (wgpu) | GPU Advantage |
|-----------|-------------------|------------|---------------|
| **Small images (<256×256)** | ★★★ Faster | ★★ Overhead dominant | CPU wins |
| **Medium images (1K-2K)** | ★★ OK | ★★★ Good | ~2-10x faster |
| **Large images (4K+)** | ★ Slow | ★★★ Excellent | 10-100x faster |
| **Batch processing** | ★ Linear scaling | ★★★ Parallel | Massive win |
| **Complex filters (convolution)** | ★ Very slow | ★★★ Parallel | 50-500x faster |
| **Simple ops (brightness)** | ★★★ Fast | ★★ Overhead | CPU often wins |

### When to Use GPU

**✅ GPU Excels At:**
- Large images (4K, 8K, panoramas)
- Complex operations (convolutions, FFT)
- Real-time video/camera processing
- Batch processing multiple images
- Operations that are memory-bound on CPU

**❌ GPU Not Worth It For:**
- Small thumbnails (<512×512)
- Simple pixel operations
- Single small image, one-time processing
- When GPU isn't available (fallback needed anyway)

### Overhead Considerations

```
┌─────────────────────────────────────────────────────────────────┐
│ GPU Processing Overhead (per operation)                         │
├─────────────────────────────────────────────────────────────────┤
│ 1. Upload data to GPU         ~0.5-5ms (size dependent)        │
│ 2. Shader compilation         ~10-100ms (first time only)      │
│ 3. Dispatch compute           ~0.01-0.1ms                      │
│ 4. GPU execution              ~0.1-50ms (operation dependent)  │
│ 5. Download results           ~0.5-5ms (size dependent)        │
├─────────────────────────────────────────────────────────────────┤
│ Total overhead: 1-10ms + execution time                        │
│ For a 4K image with complex filter: GPU saves 100ms+           │
│ For a 256×256 thumbnail: CPU is 5-10x faster                   │
└─────────────────────────────────────────────────────────────────┘
```

### Benchmarking Code

```rust
use std::time::Instant;
use image::{ImageBuffer, Rgba};

// CPU benchmark using `image` crate
fn cpu_blur(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let start = Instant::now();
    let result = imageproc::filter::gaussian_blur_f32(img, 3.0);
    println!("CPU blur: {:?}", start.elapsed());
    result
}

// GPU benchmark using wgpu
async fn gpu_blur(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let start = Instant::now();
    
    // Create output texture, bind group, dispatch...
    // (implementation details omitted for brevity)
    
    let result = execute_blur(device, queue, pipeline, texture, width, height).await;
    println!("GPU blur: {:?}", start.elapsed());
    result
}
```

---

## 6. Example Code Patterns

### Complete Image Processor Structure

```rust
use wgpu::util::DeviceExt;

pub struct GpuImageProcessor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    blur_pipeline: wgpu::ComputePipeline,
    sharpen_pipeline: wgpu::ComputePipeline,
    color_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuImageProcessor {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&Default::default());
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("No adapter");
            
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Image Processor"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            }, None)
            .await
            .expect("No device");
        
        // Create bind group layout for all shaders
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        // Create pipelines
        let blur_pipeline = Self::create_pipeline(&device, &bind_group_layout, include_str!("blur.wgsl"));
        let sharpen_pipeline = Self::create_pipeline(&device, &bind_group_layout, include_str!("sharpen.wgsl"));
        let color_pipeline = Self::create_pipeline(&device, &bind_group_layout, include_str!("color.wgsl"));
        
        Self {
            device,
            queue,
            blur_pipeline,
            sharpen_pipeline,
            color_pipeline,
            bind_group_layout,
        }
    }
    
    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        shader_source: &str,
    ) -> wgpu::ComputePipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[layout],
            push_constant_ranges: &[],
        });
        
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
    }
    
    pub async fn apply_blur(&self, input: &[u8], width: u32, height: u32) -> Vec<u8> {
        self.run_pipeline(&self.blur_pipeline, input, width, height).await
    }
    
    pub async fn apply_sharpen(&self, input: &[u8], width: u32, height: u32) -> Vec<u8> {
        self.run_pipeline(&self.sharpen_pipeline, input, width, height).await
    }
    
    async fn run_pipeline(
        &self,
        pipeline: &wgpu::ComputePipeline,
        input: &[u8],
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        // Create input texture
        let input_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Input"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &input_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            input,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        // Create output texture
        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &input_texture.create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&Default::default())
                    ),
                },
            ],
        });
        
        // Dispatch
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                (width + 7) / 8,
                (height + 7) / 8,
                1,
            );
        }
        self.queue.submit([encoder.finish()]);
        
        // Read back results (implementation from section 4)
        read_texture_data(&self.device, &self.queue, &output_texture, width, height).await
    }
}
```

### WASM Entry Point

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't init logger");
    }
    
    let processor = GpuImageProcessor::new().await;
    
    // Process images...
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn process_image(data: &[u8], width: u32, height: u32, operation: &str) -> Vec<u8> {
    let processor = GpuImageProcessor::new().await;
    
    match operation {
        "blur" => processor.apply_blur(data, width, height).await,
        "sharpen" => processor.apply_sharpen(data, width, height).await,
        _ => data.to_vec(),
    }
}
```

---

## 7. Limitations in Browser WASM Context

### WebGPU Browser Support (as of 2025)

| Browser | Status | Notes |
|---------|--------|-------|
| **Chrome 113+** | ✅ Supported | Full support |
| **Edge 113+** | ✅ Supported | Chromium-based |
| **Safari 26+** | ⚠️ Partial | Recently added, some issues |
| **Firefox** | ❌ Disabled | Behind flag, not production-ready |
| **Opera 99+** | ✅ Supported | Chromium-based |
| **Chrome Android** | ✅ Supported | Works well |
| **Safari iOS 26+** | ✅ Supported | Works |
| **Samsung Internet 24+** | ✅ Supported | Works |

### Global Support Statistics

- **~75-80%** of users have WebGPU available
- **~95%+** with WebGL2 fallback
- Desktop support is better than mobile

### WASM-Specific Limitations

1. **No Shared Memory by Default**
   - All buffer mapping involves IPC copies
   - More expensive than native
   
2. **Async Constraints**
   - All GPU operations must be async
   - Can't block on GPU operations
   
3. **Thread Limitations**
   - Web Workers can't share wgpu resources directly
   - SharedArrayBuffer requires special headers
   
4. **Storage Texture Formats**
   - WebGPU only guarantees `rgba8unorm` for storage
   - Other formats may not be writable

5. **Compute Shader Limits**
   ```
   maxComputeInvocationsPerWorkgroup: 256
   maxComputeWorkgroupSizeX: 256
   maxComputeWorkgroupSizeY: 256
   maxComputeWorkgroupSizeZ: 64
   maxComputeWorkgroupsPerDimension: 65535
   ```

### Feature Detection

```rust
// Check if we're running on WebGPU or WebGL backend
let adapter_info = adapter.get_info();
let is_webgpu = adapter_info.backend == wgpu::Backend::BrowserWebGpu;
let is_webgl = adapter_info.backend == wgpu::Backend::Gl;

// Check specific features
let supports_storage_textures = device.features().contains(
    wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
);
```

---

## 8. Fallback Strategies

### Strategy 1: Feature Detection with CPU Fallback

```rust
pub enum ImageBackend {
    Gpu(GpuImageProcessor),
    Cpu,
}

impl ImageBackend {
    pub async fn new() -> Self {
        // Try GPU first
        match Self::try_init_gpu().await {
            Some(gpu) => ImageBackend::Gpu(gpu),
            None => {
                log::warn!("GPU not available, falling back to CPU");
                ImageBackend::Cpu
            }
        }
    }
    
    async fn try_init_gpu() -> Option<GpuImageProcessor> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance.request_adapter(&Default::default()).await?;
        
        // Check for required features
        let features = adapter.features();
        if !features.contains(wgpu::Features::empty()) {
            return None;
        }
        
        Some(GpuImageProcessor::with_adapter(adapter).await.ok()?)
    }
    
    pub async fn blur(&self, data: &[u8], width: u32, height: u32) -> Vec<u8> {
        match self {
            ImageBackend::Gpu(gpu) => gpu.apply_blur(data, width, height).await,
            ImageBackend::Cpu => cpu_blur(data, width, height),
        }
    }
}

fn cpu_blur(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    
    let img: ImageBuffer<Rgba<u8>, _> = 
        ImageBuffer::from_raw(width, height, data.to_vec()).unwrap();
    
    let blurred = imageproc::filter::gaussian_blur_f32(&img, 3.0);
    blurred.into_raw()
}
```

### Strategy 2: WebGL2 Backend for Broader Support

```rust
// Cargo.toml - Enable WebGL feature
// wgpu = { version = "0.28", features = ["webgpu", "webgl"] }

pub async fn init_with_webgl_fallback() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        // Try WebGPU first, then fall back to WebGL
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
        #[cfg(not(target_arch = "wasm32"))]
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    // Request adapter with fallback
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,  // Try hardware first
            compatible_surface: None,
        })
        .await
        .or_else(|| {
            // If no hardware adapter, try software/fallback
            futures::executor::block_on(
                instance.request_adapter(&wgpu::RequestAdapterOptions {
                    force_fallback_adapter: true,
                    ..Default::default()
                })
            )
        })
        .expect("No adapter available");
    
    let info = adapter.get_info();
    log::info!("Using adapter: {} ({:?})", info.name, info.backend);
    
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            ..Default::default()
        }, None)
        .await
        .expect("Failed to create device")
}
```

### Strategy 3: Progressive Enhancement

```rust
pub struct AdaptiveImageProcessor {
    backend: ProcessingBackend,
    image_size_threshold: u32,  // Below this, use CPU
}

enum ProcessingBackend {
    GpuWebGpu(GpuImageProcessor),
    GpuWebGl(GpuImageProcessor),
    Cpu,
}

impl AdaptiveImageProcessor {
    pub async fn process(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        operation: Operation,
    ) -> Vec<u8> {
        let pixel_count = width * height;
        
        // For small images, CPU is often faster
        if pixel_count < self.image_size_threshold as u32 {
            return self.cpu_process(data, width, height, operation);
        }
        
        // Use GPU for larger images
        match &self.backend {
            ProcessingBackend::GpuWebGpu(gpu) | ProcessingBackend::GpuWebGl(gpu) => {
                gpu.process(data, width, height, operation).await
            }
            ProcessingBackend::Cpu => {
                self.cpu_process(data, width, height, operation)
            }
        }
    }
    
    fn cpu_process(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        operation: Operation,
    ) -> Vec<u8> {
        // Implement using `image` crate
        match operation {
            Operation::Blur(radius) => cpu_blur(data, width, height, radius),
            Operation::Sharpen(amount) => cpu_sharpen(data, width, height, amount),
            Operation::Brightness(value) => cpu_brightness(data, width, height, value),
        }
    }
}
```

### Strategy 4: Service Worker for Compute

```javascript
// For complex processing, offload to a Service Worker
// which can use WebGPU in a separate context

// main.js
if ('serviceWorker' in navigator && 'gpu' in navigator) {
    const registration = await navigator.serviceWorker.register('/image-worker.js');
    
    // Send image to worker
    navigator.serviceWorker.controller.postMessage({
        type: 'process',
        imageData: imageArrayBuffer,
        operation: 'blur',
        params: { radius: 5 }
    });
}
```

---

## Summary

### Quick Reference

| Need | Solution |
|------|----------|
| Basic setup | `wgpu` + `wasm-bindgen` + `wasm-pack` |
| Shaders | WGSL (always supported) |
| Workgroup size | 8×8 or 64 total (safe default) |
| Texture format | `Rgba8Unorm` (universal) |
| CPU fallback | `image` + `imageproc` crates |
| Browser support | ~75% WebGPU, ~95% with WebGL2 |

### Best Practices

1. **Always have a CPU fallback** - Not all users have WebGPU
2. **Cache pipelines** - Shader compilation is expensive
3. **Batch operations** - Minimize CPU↔GPU transfers
4. **Use appropriate workgroup sizes** - 64 is a safe default
5. **Profile on target platforms** - Performance varies significantly
6. **Handle async properly** - Especially important in WASM

### Further Reading

- [wgpu Documentation](https://docs.rs/wgpu/latest/wgpu/)
- [Learn WGPU](https://sotrh.github.io/learn-wgpu/)
- [WebGPU Fundamentals](https://webgpufundamentals.org/)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)
- [WebGPU Implementation Status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
