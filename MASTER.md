# FlowNode WebAssembly - Master Technical Specification

> **Version:** 0.1.0-draft  
> **Last Updated:** 2026-01-28  
> **Status:** Planning Phase  
> **Repo:** https://github.com/designco-agency/flownode-webassembly

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Core Systems](#3-core-systems)
4. [Node System](#4-node-system)
5. [Image Processing Pipeline](#5-image-processing-pipeline)
6. [User Interface](#6-user-interface)
7. [File I/O & Formats](#7-file-io--formats)
8. [Performance & Optimization](#8-performance--optimization)
9. [AI Integration](#9-ai-integration)
10. [Deployment Strategy](#10-deployment-strategy)
11. [Development Phases](#11-development-phases)
12. [Technical Appendices](#12-technical-appendices)

---

## 1. Executive Summary

### 1.1 Vision

FlowNode is a **professional-grade, node-based image editor** built entirely in Rust and compiled to WebAssembly. It runs in the browser with near-native performance, leveraging WebGPU for GPU-accelerated image processing.

### 1.2 Key Differentiators

| Feature | Traditional Web Apps | FlowNode |
|---------|---------------------|----------|
| **Performance** | JS-limited, 60fps struggle | Native-speed, GPU-accelerated |
| **Architecture** | DOM-based, React/Vue | Canvas-only, immediate-mode GUI |
| **Processing** | CPU-bound, slow | WebGPU compute shaders |
| **Portability** | Web only | Web + Desktop from same codebase |
| **Memory** | JS heap limits | Direct WASM linear memory |

### 1.3 Target Users

- Professional photographers and retouchers
- Digital artists and illustrators
- Video production teams (still frame work)
- Marketing teams needing quick edits
- Developers integrating image processing

### 1.4 Core Principles

1. **Non-destructive editing** - Original images never modified
2. **Real-time preview** - Changes visible instantly
3. **GPU-first** - CPU fallback only when necessary
4. **Offline-capable** - Full functionality without internet
5. **Open format** - Project files are human-readable JSON

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                               │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   index.html │  │  WASM Module │  │   Web Workers      │  │
│  │   (loader)   │  │  (main app)  │  │   (background)     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         ▼                ▼                     ▼             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    <canvas>                              ││
│  │         (Single canvas, GPU-rendered)                    ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                     WebGPU / WebGL                          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Module Structure

```
flownode-webassembly/
├── crates/
│   ├── flownode-core/        # Core data structures, no UI
│   ├── flownode-gpu/         # wgpu rendering & compute
│   ├── flownode-nodes/       # Node definitions & evaluation
│   ├── flownode-ui/          # egui UI components
│   ├── flownode-io/          # File format handlers
│   └── flownode-ai/          # AI feature integrations
├── src/
│   └── lib.rs                # WASM entry point
├── web/
│   ├── index.html
│   ├── worker.js             # Web Worker bridge
│   └── sw.js                 # Service Worker
└── native/
    └── main.rs               # Native desktop entry
```

### 2.3 Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Language** | Rust | Memory safety, performance |
| **GUI** | egui 0.30+ | Immediate-mode UI |
| **Graphics** | wgpu | WebGPU/WebGL abstraction |
| **Windowing** | eframe | Cross-platform window |
| **Serialization** | serde | JSON/binary formats |
| **Build** | Trunk | WASM bundling |
| **Testing** | criterion | Benchmarks |

---

## 3. Core Systems

### 3.1 Memory Management

**Priority:** P0 (Critical)

#### 3.1.1 Image Buffer System

```rust
pub struct ImageBuffer {
    /// Raw pixel data (RGBA f32 for HDR support)
    data: Vec<f32>,
    /// Image dimensions
    width: u32,
    height: u32,
    /// Color space
    color_space: ColorSpace,
    /// Bit depth of original
    source_depth: BitDepth,
}
```

#### 3.1.2 Memory Pools

- **Scratch buffers** - Reusable temporary buffers for operations
- **Texture cache** - GPU texture LRU cache
- **History snapshots** - Compressed delta storage

#### 3.1.3 WASM Memory Constraints

| Browser | Max WASM Memory | Practical Limit |
|---------|-----------------|-----------------|
| Chrome | 4GB | ~2GB stable |
| Firefox | 4GB | ~2GB stable |
| Safari | 4GB | ~1.5GB stable |

**Strategy:** Stream large images, tile-based processing for >100MP images.

### 3.2 Event System

**Priority:** P1

```rust
pub enum AppEvent {
    // Input
    NodeCreated { node_id: NodeId, node_type: NodeType },
    NodeDeleted { node_id: NodeId },
    ConnectionMade { from: OutputSlot, to: InputSlot },
    ParameterChanged { node_id: NodeId, param: String, value: Value },
    
    // Processing
    ProcessingStarted { node_id: NodeId },
    ProcessingComplete { node_id: NodeId, duration_ms: u64 },
    ProcessingFailed { node_id: NodeId, error: String },
    
    // UI
    ViewportPanned { delta: Vec2 },
    ViewportZoomed { factor: f32, center: Pos2 },
    SelectionChanged { nodes: Vec<NodeId> },
}
```

### 3.3 Undo/Redo System

**Priority:** P0

#### 3.3.1 Command Pattern

```rust
pub trait Command: Send + Sync {
    fn execute(&self, state: &mut AppState) -> Result<()>;
    fn undo(&self, state: &mut AppState) -> Result<()>;
    fn description(&self) -> &str;
    fn merge_with(&self, other: &dyn Command) -> Option<Box<dyn Command>>;
}
```

#### 3.3.2 History Structure

```rust
pub struct History {
    commands: Vec<Box<dyn Command>>,
    current_index: usize,
    max_history: usize,        // Default: 100
    snapshot_interval: usize,  // Full state every N commands
    snapshots: HashMap<usize, CompressedState>,
}
```

---

## 4. Node System

### 4.1 Node Architecture

**Priority:** P0

#### 4.1.1 Base Node Structure

```rust
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: Pos2,
    pub title: String,
    pub inputs: Vec<InputSlot>,
    pub outputs: Vec<OutputSlot>,
    pub parameters: HashMap<String, Parameter>,
    pub cached_output: Option<ProcessedData>,
    pub dirty: bool,
}
```

#### 4.1.2 Slot Types

```rust
pub enum DataType {
    Image,           // RGBA image buffer
    Mask,            // Single-channel mask
    Number(NumType), // f32, i32, etc.
    Color,           // RGBA color
    String,          // Text
    Curve,           // Bezier curve data
    Gradient,        // Color gradient
    Transform,       // 2D/3D transform matrix
}
```

### 4.2 Node Categories

#### 4.2.1 Input Nodes (P0)

| Node | Description | Outputs |
|------|-------------|---------|
| **Image Input** | Load from file/URL | Image |
| **Solid Color** | Generate solid color | Image |
| **Gradient** | Linear/radial gradient | Image |
| **Noise** | Perlin/Simplex/Worley | Image, Mask |
| **Checkerboard** | Pattern generator | Image |
| **Text** | Render text to image | Image, Mask |

#### 4.2.2 Adjustment Nodes (P0)

| Node | Parameters | Description |
|------|------------|-------------|
| **Brightness/Contrast** | brightness: -1..1, contrast: -1..1 | Basic levels |
| **Levels** | black, white, gamma per channel | Histogram adjust |
| **Curves** | RGB + individual curves | Pro color grading |
| **Hue/Saturation** | hue: -180..180, sat: -1..1, light: -1..1 | HSL adjust |
| **Color Balance** | shadows/mids/highlights RGB | Color correction |
| **Exposure** | exposure: -5..5, offset, gamma | HDR-aware |
| **Vibrance** | vibrance: -1..1, saturation: -1..1 | Smart saturation |

#### 4.2.3 Filter Nodes (P0-P1)

| Node | Priority | Parameters |
|------|----------|------------|
| **Blur (Gaussian)** | P0 | radius: 0..500 |
| **Blur (Box)** | P0 | radius: 0..500 |
| **Blur (Motion)** | P1 | angle, distance |
| **Blur (Radial)** | P1 | center, amount |
| **Blur (Lens/Bokeh)** | P2 | radius, shape, highlights |
| **Sharpen** | P0 | amount, radius, threshold |
| **Unsharp Mask** | P0 | amount, radius, threshold |
| **High Pass** | P1 | radius |
| **Noise Reduction** | P2 | luminance, color, detail |

#### 4.2.4 Transform Nodes (P0-P1)

| Node | Priority | Description |
|------|----------|-------------|
| **Resize** | P0 | Scale with various algorithms |
| **Crop** | P0 | Rectangular crop |
| **Rotate** | P0 | Free rotation with interpolation |
| **Flip** | P0 | Horizontal/vertical flip |
| **Perspective** | P1 | 4-point perspective correction |
| **Lens Correction** | P2 | Distortion, CA, vignette |
| **Liquify** | P3 | Mesh-based warping |

#### 4.2.5 Combine Nodes (P0)

| Node | Description | Inputs |
|------|-------------|--------|
| **Blend** | Layer blending modes | Base, Blend, Mask |
| **Merge** | Composite multiple | Multiple images |
| **Mask Apply** | Apply mask to alpha | Image, Mask |
| **Channel Mixer** | Remix RGB channels | Image |
| **Split Channels** | Output R, G, B, A separately | Image |
| **Combine Channels** | Merge separate channels | R, G, B, A |

#### 4.2.6 Selection/Mask Nodes (P1)

| Node | Description |
|------|-------------|
| **Color Range** | Select by color similarity |
| **Luminosity Mask** | Select by brightness |
| **Edge Detection** | Canny/Sobel edge mask |
| **Threshold** | Binary mask from image |
| **Feather** | Blur mask edges |
| **Invert Mask** | Invert selection |

#### 4.2.7 Output Nodes (P0)

| Node | Description |
|------|-------------|
| **Viewer** | Preview in main canvas |
| **Export** | Save to file |
| **Compare** | Side-by-side/overlay comparison |

### 4.3 Blend Modes

**Priority:** P0

| Mode | Formula | Use Case |
|------|---------|----------|
| Normal | B | Basic compositing |
| Multiply | A × B | Darken, shadows |
| Screen | 1-(1-A)(1-B) | Lighten, highlights |
| Overlay | A<0.5 ? 2AB : 1-2(1-A)(1-B) | Contrast |
| Soft Light | Complex | Subtle contrast |
| Hard Light | Overlay flipped | Strong contrast |
| Color Dodge | A/(1-B) | Brighten dramatically |
| Color Burn | 1-(1-A)/B | Darken dramatically |
| Difference | |A-B| | Find differences |
| Exclusion | A+B-2AB | Softer difference |
| Hue | HSL(B.H, A.S, A.L) | Change hue only |
| Saturation | HSL(A.H, B.S, A.L) | Change saturation |
| Color | HSL(B.H, B.S, A.L) | Change hue+sat |
| Luminosity | HSL(A.H, A.S, B.L) | Change brightness |

### 4.4 Graph Evaluation

**Priority:** P0

#### 4.4.1 Topological Sort

```rust
pub fn evaluate_order(graph: &Graph) -> Vec<NodeId> {
    // Kahn's algorithm for topological sort
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    let mut result: Vec<NodeId> = Vec::new();
    
    // Initialize in-degrees
    for node in graph.nodes() {
        let degree = node.inputs.iter()
            .filter(|i| i.connection.is_some())
            .count();
        in_degree.insert(node.id, degree);
        if degree == 0 {
            queue.push_back(node.id);
        }
    }
    
    // Process
    while let Some(node_id) = queue.pop_front() {
        result.push(node_id);
        for output in graph.node(node_id).outputs.iter() {
            for conn in output.connections.iter() {
                let entry = in_degree.get_mut(&conn.target_node).unwrap();
                *entry -= 1;
                if *entry == 0 {
                    queue.push_back(conn.target_node);
                }
            }
        }
    }
    
    result
}
```

#### 4.4.2 Dirty Propagation

When a node changes:
1. Mark node as dirty
2. Mark all downstream nodes as dirty
3. Re-evaluate only dirty nodes in topological order

#### 4.4.3 Caching Strategy

```rust
pub enum CachePolicy {
    Always,      // Always cache (output nodes)
    Smart,       // Cache if > N downstream nodes
    Never,       // Never cache (simple operations)
    Invalidate,  // Cache but invalidate on memory pressure
}
```

---

## 5. Image Processing Pipeline

### 5.1 GPU Architecture (wgpu)

**Priority:** P0

#### 5.1.1 Pipeline Structure

```rust
pub struct GpuPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,
    
    // Compute pipelines for each operation
    pipelines: HashMap<String, wgpu::ComputePipeline>,
    
    // Bind group layouts
    image_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
    
    // Texture cache
    texture_cache: LruCache<TextureId, wgpu::Texture>,
    
    // Staging buffers for CPU<->GPU transfer
    staging_belt: wgpu::util::StagingBelt,
}
```

#### 5.1.2 Compute Shader Example (Gaussian Blur)

```wgsl
// blur.wgsl
@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: BlurParams;

struct BlurParams {
    radius: u32,
    sigma: f32,
    direction: vec2<f32>, // (1,0) for horizontal, (0,1) for vertical
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;
    
    for (var i: i32 = -i32(params.radius); i <= i32(params.radius); i++) {
        let offset = vec2<i32>(gid.xy) + vec2<i32>(params.direction * f32(i));
        let clamped = clamp(offset, vec2<i32>(0), vec2<i32>(dims) - 1);
        
        let weight = exp(-f32(i * i) / (2.0 * params.sigma * params.sigma));
        color += textureLoad(input_tex, clamped, 0) * weight;
        weight_sum += weight;
    }
    
    textureStore(output_tex, vec2<i32>(gid.xy), color / weight_sum);
}
```

#### 5.1.3 Two-Pass Separable Convolution

For efficiency, large kernels use separable 2-pass approach:
1. Horizontal pass (blur rows)
2. Vertical pass (blur columns)

Reduces O(n²) to O(2n) for kernel size n.

### 5.2 CPU Fallback

**Priority:** P1

For browsers without WebGPU:

```rust
#[cfg(feature = "cpu-fallback")]
pub fn blur_cpu(input: &ImageBuffer, radius: u32) -> ImageBuffer {
    // Use rayon for parallel processing
    use rayon::prelude::*;
    
    let kernel = generate_gaussian_kernel(radius);
    let mut output = ImageBuffer::new(input.width, input.height);
    
    // Parallel row processing
    output.rows_mut().par_iter_mut().enumerate().for_each(|(y, row)| {
        for x in 0..input.width {
            // Convolve...
        }
    });
    
    output
}
```

### 5.3 Color Management

**Priority:** P1

#### 5.3.1 Supported Color Spaces

| Space | Internal Format | Use Case |
|-------|-----------------|----------|
| sRGB | f32 linear | Web display |
| Linear RGB | f32 | Processing |
| Adobe RGB | f32 linear | Print workflow |
| ProPhoto RGB | f32 linear | Wide gamut |
| Display P3 | f32 linear | Modern displays |
| ACES | f32 linear | HDR/film |

#### 5.3.2 ICC Profile Handling

```rust
pub struct ColorProfile {
    name: String,
    icc_data: Vec<u8>,
    to_xyz: ColorMatrix,
    from_xyz: ColorMatrix,
    tone_curve: ToneCurve,
}

impl ColorProfile {
    pub fn convert_to(&self, target: &ColorProfile, pixel: [f32; 3]) -> [f32; 3] {
        let xyz = self.to_xyz.transform(pixel);
        target.from_xyz.transform(xyz)
    }
}
```

### 5.4 HDR Support

**Priority:** P2

- Internal processing in linear f32 (unlimited dynamic range)
- Tone mapping for SDR display (Reinhard, ACES, Filmic)
- EXR/HDR file format support
- Exposure bracketing merge

---

## 6. User Interface

### 6.1 Layout Structure

**Priority:** P0

```
┌─────────────────────────────────────────────────────────────────┐
│ Menu Bar                                            [_][□][×]   │
├──────────┬──────────────────────────────────────────┬───────────┤
│          │                                          │           │
│  Node    │                                          │ Properties│
│  Library │           Canvas (Node Graph)            │   Panel   │
│          │                                          │           │
│  ────────│                                          │───────────│
│          │                                          │           │
│  Layers  │                                          │  History  │
│  Panel   │                                          │   Panel   │
│          │                                          │           │
├──────────┴──────────────────────────────────────────┴───────────┤
│ Status Bar: [Zoom: 100%] [Nodes: 12] [GPU: wgpu] [Mem: 1.2GB]   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Panel Components

#### 6.2.1 Node Library (P0)

- Categorized node list (collapsible)
- Search/filter functionality
- Drag-to-create interaction
- Favorites/recent nodes
- Custom node templates

#### 6.2.2 Properties Panel (P0)

- Dynamic UI based on selected node
- Parameter widgets: sliders, color pickers, curves
- Reset to default buttons
- Parameter linking/expressions
- Presets dropdown

#### 6.2.3 Canvas/Graph View (P0)

- Infinite pan with smooth scrolling
- Zoom: 10% - 400%
- Minimap overlay
- Grid snapping (optional)
- Multi-select with box selection
- Connection preview while dragging
- Bezier curves for connections

#### 6.2.4 History Panel (P1)

- Linear undo/redo list
- Snapshot markers
- Branch history (optional P3)
- Memory usage per snapshot

### 6.3 Interaction Patterns

**Priority:** P0

| Action | Mouse | Keyboard |
|--------|-------|----------|
| Pan | Middle-drag / Space+drag | Arrow keys |
| Zoom | Scroll wheel | +/- |
| Select | Click | - |
| Multi-select | Ctrl+click / Box drag | - |
| Delete | - | Delete/Backspace |
| Duplicate | - | Ctrl+D |
| Cut/Copy/Paste | - | Ctrl+X/C/V |
| Undo/Redo | - | Ctrl+Z / Ctrl+Shift+Z |
| Create node | Double-click | Tab (opens search) |
| Connect | Drag from slot | - |

### 6.4 Themes

**Priority:** P2

```rust
pub struct Theme {
    // Background
    canvas_bg: Color32,
    panel_bg: Color32,
    
    // Nodes
    node_bg: Color32,
    node_header_colors: HashMap<NodeCategory, Color32>,
    node_selected_outline: Color32,
    
    // Connections
    connection_colors: HashMap<DataType, Color32>,
    connection_width: f32,
    
    // Text
    font_family: String,
    font_sizes: FontSizes,
    text_color: Color32,
}
```

---

## 7. File I/O & Formats

### 7.1 Native Project Format (.flownode)

**Priority:** P0

```json
{
  "version": "1.0",
  "name": "My Project",
  "created": "2026-01-28T12:00:00Z",
  "modified": "2026-01-28T14:30:00Z",
  "canvas": {
    "pan": [0, 0],
    "zoom": 1.0
  },
  "nodes": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "type": "ImageInput",
      "position": [100, 200],
      "parameters": {
        "file": "images/photo.jpg"
      }
    }
  ],
  "connections": [
    {
      "from": { "node": "...", "slot": 0 },
      "to": { "node": "...", "slot": 0 }
    }
  ],
  "assets": {
    "images/photo.jpg": {
      "hash": "sha256:abc123...",
      "size": 1024000,
      "embedded": false
    }
  }
}
```

### 7.2 Import Formats

**Priority Levels:**

| Format | Priority | Library |
|--------|----------|---------|
| JPEG | P0 | image-rs |
| PNG | P0 | image-rs |
| WebP | P0 | image-rs |
| TIFF | P1 | image-rs |
| BMP | P1 | image-rs |
| GIF | P1 | image-rs |
| PSD | P2 | psd-rs |
| EXR | P2 | exr-rs |
| RAW (CR2, NEF, ARW) | P2 | rawloader |
| SVG | P3 | resvg |
| HEIC | P3 | libheif bindings |

### 7.3 Export Formats

| Format | Priority | Options |
|--------|----------|---------|
| JPEG | P0 | Quality 1-100, progressive |
| PNG | P0 | Compression level, bit depth |
| WebP | P0 | Lossy/lossless, quality |
| TIFF | P1 | Compression, bit depth |
| EXR | P2 | Compression, channels |
| PDF | P3 | Resolution, color space |

### 7.4 Batch Processing

**Priority:** P2

```rust
pub struct BatchJob {
    input_files: Vec<PathBuf>,
    graph: Graph,                    // Node graph to apply
    output_template: String,         // e.g., "{name}_edited.{ext}"
    output_format: ExportFormat,
    output_directory: PathBuf,
    parallel_jobs: usize,
}
```

---

## 8. Performance & Optimization

### 8.1 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Initial Load** | <3s | Time to interactive |
| **WASM Size** | <5MB | Compressed bundle |
| **Frame Rate** | 60fps | UI interactions |
| **Node Add** | <16ms | Single node creation |
| **Blur (1000px radius)** | <100ms | 4K image, GPU |
| **Graph Eval** | <1s | 50 nodes, 4K image |
| **Memory** | <2GB | 4K editing session |

### 8.2 Optimization Strategies

#### 8.2.1 WASM Binary Size

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller panic handling
strip = true         # Strip symbols
```

**Additional:**
- Tree shaking unused code
- Lazy load node implementations
- Separate chunks for heavy features (AI nodes)

#### 8.2.2 Graph Culling

```rust
// Only render visible nodes
fn visible_nodes(graph: &Graph, viewport: Rect, zoom: f32) -> Vec<NodeId> {
    let expanded = viewport.expand(100.0 / zoom); // Buffer for connections
    graph.nodes()
        .filter(|n| expanded.contains_rect(n.bounding_rect()))
        .map(|n| n.id)
        .collect()
}
```

#### 8.2.3 Texture Streaming

For images larger than GPU memory:
1. Load thumbnail first
2. Stream full resolution tiles on demand
3. Use mipmaps for zoomed-out views

### 8.3 Web Workers

**Priority:** P1

```javascript
// worker.js
import init, { process_heavy_operation } from './flownode_canvas.js';

self.onmessage = async (e) => {
    await init();
    const result = process_heavy_operation(e.data);
    self.postMessage(result);
};
```

Use cases:
- Histogram calculation
- AI inference
- Batch export
- Large file parsing

---

## 9. AI Integration

### 9.1 Built-in AI Nodes

**Priority:** P2-P3

| Node | Priority | Description |
|------|----------|-------------|
| **Background Remove** | P2 | Remove background (U-2-Net) |
| **Upscale** | P2 | Super resolution (ESRGAN) |
| **Denoise** | P2 | AI noise reduction |
| **Content-Aware Fill** | P3 | Inpainting |
| **Style Transfer** | P3 | Apply artistic styles |
| **Face Detection** | P3 | Detect and mask faces |
| **Object Detection** | P3 | Detect and mask objects |
| **Text-to-Image** | P3 | Stable Diffusion integration |

### 9.2 Model Integration

```rust
pub trait AiModel: Send + Sync {
    fn name(&self) -> &str;
    fn input_requirements(&self) -> ModelInput;
    fn run(&self, input: &ImageBuffer) -> Result<ModelOutput>;
    fn supports_gpu(&self) -> bool;
}

pub struct OnnxModel {
    session: ort::Session,
    input_name: String,
    output_name: String,
}
```

### 9.3 External AI Services

**Priority:** P3

- OpenAI DALL-E API integration
- Stability AI API integration
- Local Stable Diffusion via WebSocket bridge
- Custom model server protocol

---

## 10. Deployment Strategy

### 10.1 Build Pipeline

```yaml
# .github/workflows/deploy.yml
name: Build and Deploy

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Install Trunk
        run: cargo install trunk
      
      - name: Build WASM
        run: trunk build --release
      
      - name: Optimize WASM
        run: wasm-opt -Oz dist/*.wasm -o dist/flownode_optimized.wasm
      
      - name: Deploy to Vercel
        uses: vercel/action@v1
        with:
          vercel-token: ${{ secrets.VERCEL_TOKEN }}
```

### 10.2 CDN Configuration

```json
// vercel.json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" },
        { "key": "Cross-Origin-Opener-Policy", "value": "same-origin" }
      ]
    },
    {
      "source": "/(.*).wasm",
      "headers": [
        { "key": "Content-Type", "value": "application/wasm" },
        { "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }
      ]
    }
  ]
}
```

### 10.3 Progressive Loading

1. **Skeleton UI** (HTML/CSS) - instant
2. **Core WASM** (~1MB) - basic UI
3. **Node modules** (lazy) - on first use
4. **AI models** (lazy) - on demand

---

## 11. Development Phases

### Phase 1: Foundation (4 weeks)

**Goal:** Basic working node editor

| Task | Duration | Priority |
|------|----------|----------|
| Project setup, CI/CD | 3 days | P0 |
| Core data structures | 5 days | P0 |
| egui integration | 5 days | P0 |
| Basic node graph (create, connect, delete) | 7 days | P0 |
| Image input node | 3 days | P0 |
| Output/viewer node | 2 days | P0 |
| File save/load (.flownode) | 3 days | P0 |

### Phase 2: Processing (4 weeks)

**Goal:** GPU-accelerated image processing

| Task | Duration | Priority |
|------|----------|----------|
| wgpu integration | 5 days | P0 |
| Compute shader framework | 5 days | P0 |
| Adjustment nodes (brightness, contrast, levels) | 5 days | P0 |
| Filter nodes (blur, sharpen) | 5 days | P0 |
| Blend modes | 3 days | P0 |
| Color management basics | 3 days | P1 |

### Phase 3: Polish (4 weeks)

**Goal:** Production-ready core

| Task | Duration | Priority |
|------|----------|----------|
| Undo/redo system | 4 days | P0 |
| History panel | 2 days | P1 |
| Node search/filter | 2 days | P1 |
| Properties panel improvements | 3 days | P1 |
| Curves node | 4 days | P1 |
| Transform nodes | 5 days | P1 |
| Export formats | 3 days | P1 |
| Performance optimization | 3 days | P0 |

### Phase 4: Advanced (6 weeks)

**Goal:** Professional features

| Task | Duration | Priority |
|------|----------|----------|
| Selection/mask nodes | 5 days | P1 |
| Advanced filters (lens blur, etc.) | 5 days | P2 |
| PSD import | 5 days | P2 |
| Batch processing | 4 days | P2 |
| AI: Background remove | 5 days | P2 |
| AI: Upscale | 4 days | P2 |
| Themes/customization | 3 days | P2 |
| Documentation | 5 days | P1 |

### Phase 5: Scale (Ongoing)

**Goal:** Enterprise features

- Collaboration (multi-user editing)
- Plugin API
- Additional AI integrations
- Desktop app distribution
- Mobile companion app

---

## 12. Technical Appendices

### Appendix A: Blend Mode Formulas

[Detailed mathematical formulas for all blend modes]

### Appendix B: WGSL Shader Library

[Complete compute shader implementations]

### Appendix C: Keyboard Shortcuts Reference

[Full shortcut mapping]

### Appendix D: API Documentation

[Public API for plugin developers]

### Appendix E: Benchmarks

[Performance benchmark results]

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0-draft | 2026-01-28 | DesigncoBot + Gemini + Claude | Initial specification |

---

## Related Research Documents

Detailed research documents are maintained separately:

| Document | Description |
|----------|-------------|
| `memory/research-rust-wasm-deployment.md` | WASM optimization, code splitting, threading, deployment |
| `memory/research-wgpu-image-processing.md` | GPU compute shaders, texture handling *(in progress)* |
| `memory/research-node-graph-patterns.md` | Graph evaluation, undo/redo, serialization *(in progress)* |
| `memory/research-image-editing-architecture.md` | Industry patterns from Photoshop/GIMP *(in progress)* |

---

## Browser Compatibility Summary

| Feature | Coverage | Baseline | Enhanced |
|---------|----------|----------|----------|
| **WebGL2** | 97% | ✅ Required | - |
| **WebGPU** | 75% | - | ✅ Preferred |
| **WASM Threading** | 92% | - | ✅ Optional |
| **WASM SIMD** | 91% | - | ✅ Optional |

**Strategy:** Build multiple variants, feature-detect at runtime, progressively enhance.

---

*This document is a living specification. Updates will be made as development progresses.*
