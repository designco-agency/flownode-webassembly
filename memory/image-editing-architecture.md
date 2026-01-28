# Professional Image Editing Software Architecture
## Technical Specification Document

---

## Table of Contents
1. [Node-Based Systems Architecture](#1-node-based-systems-architecture)
2. [Common Image Processing Node Types](#2-common-image-processing-node-types)
3. [Layer Blending Modes & Mathematical Formulas](#3-layer-blending-modes--mathematical-formulas)
4. [Non-Destructive Editing Patterns](#4-non-destructive-editing-patterns)
5. [History/Snapshot Systems](#5-historysnapshot-systems)
6. [Color Management](#6-color-management)
7. [File Format Support](#7-file-format-support)
8. [Batch Processing Pipelines](#8-batch-processing-pipelines)
9. [Plugin/Extension Architectures](#9-pluginextension-architectures)
10. [AI-Powered Features](#10-ai-powered-features)

---

## 1. Node-Based Systems Architecture

### 1.1 Overview

Professional image editors use different architectural approaches for processing images:

| Application | Architecture Type | Description |
|-------------|------------------|-------------|
| **Adobe Photoshop** | Layer-based with Smart Objects | Traditional layer stack with non-destructive smart filters |
| **GIMP** | GEGL node-based backend | Directed Acyclic Graph (DAG) for image operations |
| **Affinity Photo** | Layer-based with live processing | Real-time 60fps pan/zoom with non-destructive editing |
| **DaVinci Resolve** | Full node-based compositor | Complete node graph for color grading and compositing |
| **Nuke** | Industry-standard node compositor | Professional VFX node-based workflow |

### 1.2 GEGL (Generic Graphics Library) - Node Architecture

GEGL provides the most documented open-source node-based image processing system:

```
┌─────────────────────────────────────────────────────┐
│                    GEGL Graph                        │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐         │
│  │  Load   │───▶│ Gaussian│───▶│ Levels  │───▶ Out │
│  │  Image  │    │  Blur   │    │ Adjust  │         │
│  └─────────┘    └─────────┘    └─────────┘         │
│       │              │              │               │
│    (source)     (filter op)    (adjust op)         │
└─────────────────────────────────────────────────────┘
```

**Key Characteristics:**
- **Directed Acyclic Graph (DAG)**: Operations form nodes, images flow as edges
- **On-demand processing**: Work computed only when required
- **Meta-operations**: Complex operations built from primitives (e.g., unsharp mask = add + multiply + subtract + gaussian blur)
- **Arbitrary color support**: Via `babl` library for color-space conversions
- **OpenCL acceleration**: GPU-accelerated operations available

### 1.3 Node Data Model

```typescript
interface ProcessingNode {
  id: string;
  type: NodeType;
  inputs: NodeConnection[];
  outputs: NodeConnection[];
  parameters: Record<string, ParameterValue>;
  cached: boolean;
  region: BoundingBox;  // Region of interest for lazy evaluation
}

interface NodeConnection {
  nodeId: string;
  portName: string;
  dataType: 'image' | 'mask' | 'value' | 'color';
}

interface ProcessingGraph {
  nodes: Map<string, ProcessingNode>;
  outputNode: string;
  evaluate(region: BoundingBox): ImageBuffer;
}
```

### 1.4 Evaluation Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Pull-based (Lazy)** | Output requests data from inputs recursively | Preview rendering, partial updates |
| **Push-based (Eager)** | Changes propagate forward through graph | Real-time filters, video |
| **Tiled evaluation** | Image divided into tiles, processed independently | Large image support, parallel processing |
| **Cached evaluation** | Intermediate results stored | Interactive editing, repeated operations |

---

## 2. Common Image Processing Node Types

### 2.1 Source/Input Nodes

| Node Type | Parameters | Description |
|-----------|------------|-------------|
| **Image Source** | `path`, `colorSpace`, `bitDepth` | Load image from file |
| **Color Generator** | `color`, `width`, `height` | Solid color fill |
| **Gradient Generator** | `type`, `colors[]`, `positions[]` | Linear/radial gradients |
| **Noise Generator** | `type`, `seed`, `scale`, `octaves` | Perlin, simplex, voronoi noise |
| **Checker Pattern** | `size`, `color1`, `color2` | Checkered pattern |

### 2.2 Filter/Effect Nodes

```typescript
// Blur family
interface GaussianBlur {
  radius: number;        // 0.1 - 1000 pixels
  sigmaX?: number;       // Optional asymmetric
  sigmaY?: number;
}

interface MotionBlur {
  angle: number;         // 0 - 360 degrees
  distance: number;      // Blur distance in pixels
}

interface RadialBlur {
  centerX: number;       // 0.0 - 1.0 normalized
  centerY: number;
  amount: number;
  type: 'spin' | 'zoom';
}

// Sharpen family
interface UnsharpMask {
  amount: number;        // 1 - 500%
  radius: number;        // 0.1 - 250 pixels
  threshold: number;     // 0 - 255 levels
}

interface HighPass {
  radius: number;
  preserveColor: boolean;
}
```

### 2.3 Adjustment Nodes

| Node | Parameters | Range |
|------|------------|-------|
| **Brightness/Contrast** | `brightness`, `contrast` | -100 to +100 |
| **Levels** | `inputBlack`, `inputWhite`, `gamma`, `outputBlack`, `outputWhite` | 0-255, 0.1-10.0 |
| **Curves** | `controlPoints[]` per channel | Array of (x,y) pairs |
| **Hue/Saturation** | `hue`, `saturation`, `lightness` | -180 to +180, -100 to +100 |
| **Color Balance** | `shadows`, `midtones`, `highlights` | RGB triplets -100 to +100 |
| **Exposure** | `exposure`, `offset`, `gamma` | -5 to +5 EV |
| **Vibrance** | `vibrance`, `saturation` | -100 to +100 |
| **Selective Color** | `colors[color].c/m/y/k` | -100 to +100 per channel |

### 2.4 Transform Nodes

```typescript
interface Transform {
  translateX: number;    // Pixels
  translateY: number;
  scaleX: number;        // 0.01 - 100.0
  scaleY: number;
  rotation: number;      // Degrees
  skewX: number;
  skewY: number;
  anchorX: number;       // Normalized 0-1
  anchorY: number;
  interpolation: 'nearest' | 'bilinear' | 'bicubic' | 'lanczos';
}

interface Perspective {
  topLeft: Point2D;
  topRight: Point2D;
  bottomLeft: Point2D;
  bottomRight: Point2D;
}

interface LensCorrection {
  distortion: number;      // Barrel/pincushion
  chromaticAberration: {
    red: number;
    blue: number;
  };
  vignetting: number;
}
```

### 2.5 Composite/Merge Nodes

| Node | Inputs | Parameters |
|------|--------|------------|
| **Blend** | `base`, `layer` | `mode`, `opacity`, `mask` |
| **Merge** | `inputs[]` | `operation`, `alpha` |
| **Mask** | `image`, `mask` | `invert`, `feather` |
| **Alpha Over** | `foreground`, `background` | `opacity` |

---

## 3. Layer Blending Modes & Mathematical Formulas

All formulas assume normalized values where `a` = base layer (0.0-1.0) and `b` = blend layer (0.0-1.0).

### 3.1 Normal/Dissolve

| Mode | Formula | Description |
|------|---------|-------------|
| **Normal** | `f(a,b) = b` | Top layer covers bottom |
| **Dissolve** | Random pixel selection | Dithered blend based on opacity |

### 3.2 Darken Modes

```
Darken:      f(a,b) = min(a, b)
Multiply:    f(a,b) = a × b
Color Burn:  f(a,b) = 1 - (1-a)/b        [clamp 0-1]
Linear Burn: f(a,b) = a + b - 1          [clamp 0-1]
Darker Color: Compare luminosity, use darker pixel
```

### 3.3 Lighten Modes

```
Lighten:      f(a,b) = max(a, b)
Screen:       f(a,b) = 1 - (1-a)(1-b)
Color Dodge:  f(a,b) = a / (1-b)         [clamp 0-1]
Linear Dodge: f(a,b) = a + b             [clamp 0-1]
Lighter Color: Compare luminosity, use lighter pixel
```

### 3.4 Contrast Modes

```typescript
// Overlay - combination of Multiply and Screen
function overlay(a: number, b: number): number {
  if (a < 0.5) {
    return 2 * a * b;
  } else {
    return 1 - 2 * (1 - a) * (1 - b);
  }
}

// Soft Light (Photoshop formula)
function softLight(a: number, b: number): number {
  if (b <= 0.5) {
    return a - (1 - 2*b) * a * (1 - a);
  } else {
    const d = a <= 0.25 
      ? ((16*a - 12) * a + 4) * a 
      : Math.sqrt(a);
    return a + (2*b - 1) * (d - a);
  }
}

// Hard Light - Overlay with layers swapped
function hardLight(a: number, b: number): number {
  return overlay(b, a);
}

// Vivid Light
function vividLight(a: number, b: number): number {
  if (b <= 0.5) {
    return colorBurn(a, 2*b);
  } else {
    return colorDodge(a, 2*(b - 0.5));
  }
}

// Linear Light
function linearLight(a: number, b: number): number {
  return clamp(a + 2*b - 1, 0, 1);
}

// Pin Light
function pinLight(a: number, b: number): number {
  if (b <= 0.5) {
    return Math.min(a, 2*b);
  } else {
    return Math.max(a, 2*b - 1);
  }
}

// Hard Mix
function hardMix(a: number, b: number): number {
  return a + b >= 1 ? 1 : 0;
}
```

### 3.5 Difference Modes

```
Difference:  f(a,b) = |a - b|
Exclusion:   f(a,b) = a + b - 2ab
Subtract:    f(a,b) = a - b             [clamp 0-1]
Divide:      f(a,b) = a / b             [clamp 0-1]
```

### 3.6 Component Modes (HSL-based)

These modes operate in a perceptual color space with Hue, Saturation, and Luminosity:

| Mode | Takes from Blend Layer | Takes from Base Layer |
|------|------------------------|----------------------|
| **Hue** | Hue | Saturation, Luminosity |
| **Saturation** | Saturation | Hue, Luminosity |
| **Color** | Hue, Saturation | Luminosity |
| **Luminosity** | Luminosity | Hue, Saturation |

---

## 4. Non-Destructive Editing Patterns

### 4.1 Core Principles

Non-destructive editing preserves original image data by storing edit operations separately:

```typescript
interface NonDestructiveDocument {
  originalSource: ImageSource;
  editStack: EditOperation[];
  
  // Rendered on-demand
  render(): ImageBuffer;
  
  // Modifications don't touch original
  addEdit(op: EditOperation): void;
  removeEdit(index: number): void;
  reorderEdit(from: number, to: number): void;
}

interface EditOperation {
  id: string;
  type: string;
  parameters: Record<string, any>;
  enabled: boolean;
  mask?: MaskData;
  blendMode: BlendMode;
  opacity: number;
}
```

### 4.2 Implementation Strategies

| Strategy | Applications | Description |
|----------|--------------|-------------|
| **Adjustment Layers** | Photoshop, Affinity | Separate layer storing only adjustments |
| **Smart Objects** | Photoshop | Embedded linked file with transformations |
| **Smart Filters** | Photoshop | Filters applied to Smart Objects |
| **Live Filters** | Affinity Photo | Real-time filter preview and editing |
| **Edit Decision List (EDL)** | Lightroom, Capture One | Text-based list of operations |
| **Node Graph** | GEGL/GIMP, DaVinci | Full DAG of operations |

### 4.3 Smart Object Architecture

```typescript
interface SmartObject {
  linkedFile?: string;           // External file reference
  embeddedData?: ArrayBuffer;    // Or embedded copy
  
  transforms: Transform[];       // Non-destructive transforms
  filters: SmartFilter[];        // Applied filters
  
  // Cached rendered result
  cachedOutput?: {
    data: ImageBuffer;
    renderParams: RenderParams;
    timestamp: number;
  };
  
  updateSource(): void;          // Re-link or update embedded
  editContents(): void;          // Open in new window for editing
}

interface SmartFilter {
  filterType: string;
  parameters: Record<string, any>;
  blendOptions: {
    mode: BlendMode;
    opacity: number;
  };
  mask?: MaskData;
  enabled: boolean;
}
```

### 4.4 Parametric Editing (RAW Development)

```typescript
interface RAWDevelopSettings {
  // Basic adjustments
  exposure: number;              // -5.0 to +5.0 EV
  contrast: number;              // -100 to +100
  highlights: number;            // -100 to +100
  shadows: number;               // -100 to +100
  whites: number;                // -100 to +100
  blacks: number;                // -100 to +100
  
  // White balance
  temperature: number;           // 2000K - 50000K
  tint: number;                  // -150 to +150 (green-magenta)
  
  // Tone curve
  toneCurve: {
    lights: number;
    darks: number;
    shadows: number;
    highlights: number;
  };
  
  // Color
  vibrance: number;
  saturation: number;
  hslAdjustments: HSLRange[];
  
  // Detail
  sharpening: SharpeningParams;
  noiseReduction: NoiseParams;
  
  // Lens corrections
  profileCorrection: boolean;
  manualCorrections: LensCorrection;
  
  // Transform
  perspectiveCorrection: PerspectiveParams;
  crop: CropParams;
}
```

---

## 5. History/Snapshot Systems

### 5.1 Linear History Stack

```typescript
interface HistorySystem {
  states: HistoryState[];
  currentIndex: number;
  maxStates: number;             // Memory limit
  
  pushState(state: HistoryState): void;
  undo(): HistoryState | null;
  redo(): HistoryState | null;
  jumpTo(index: number): void;
  createSnapshot(name: string): Snapshot;
}

interface HistoryState {
  id: string;
  timestamp: number;
  actionName: string;
  documentState: DocumentSnapshot;
  
  // For memory efficiency
  deltaFromPrevious?: StateDelta;
  fullSnapshot?: DocumentSnapshot;
}

interface Snapshot {
  id: string;
  name: string;
  thumbnail: ImageBuffer;
  fullState: DocumentSnapshot;
  createdAt: number;
}
```

### 5.2 Delta-Based History (Memory Efficient)

```typescript
interface StateDelta {
  type: 'layer' | 'adjustment' | 'mask' | 'transform';
  layerId?: string;
  
  // Store only what changed
  changedRegion?: BoundingBox;
  previousData?: ArrayBuffer;
  newData?: ArrayBuffer;
  
  // Or for parameter changes
  parameterChanges?: {
    path: string;
    oldValue: any;
    newValue: any;
  }[];
}

// Efficient undo via delta application
function applyDeltaReverse(
  currentState: DocumentSnapshot,
  delta: StateDelta
): DocumentSnapshot {
  // Restore previous state from delta
}
```

### 5.3 Non-Linear History (Photoshop Model)

```typescript
interface NonLinearHistory {
  // Tree structure allows branching
  rootState: HistoryNode;
  currentNode: HistoryNode;
  
  // Named snapshots independent of history
  snapshots: Map<string, Snapshot>;
}

interface HistoryNode {
  state: HistoryState;
  parent: HistoryNode | null;
  children: HistoryNode[];        // Branches from this point
  
  // Visual timeline position
  timestamp: number;
}
```

---

## 6. Color Management

### 6.1 ICC Profile System

```typescript
interface ICCProfile {
  version: '2.4' | '4.4';
  profileClass: 
    | 'input'           // Scanner, camera
    | 'display'         // Monitor
    | 'output'          // Printer
    | 'deviceLink'      // Direct device-to-device
    | 'colorSpace'      // Color space conversion
    | 'abstract'        // Effects
    | 'namedColor';     // Spot colors
    
  colorSpace: 'RGB' | 'CMYK' | 'Lab' | 'XYZ' | 'Gray';
  pcs: 'Lab' | 'XYZ';   // Profile Connection Space
  
  // Rendering intents
  renderingIntents: {
    perceptual: TransformTable;
    relativeColorimetric: TransformTable;
    saturation: TransformTable;
    absoluteColorimetric: TransformTable;
  };
  
  // White point (D50 for ICC)
  mediaWhitePoint: [number, number, number];  // XYZ
}
```

### 6.2 Color Space Definitions

| Color Space | Gamut | Typical Use | Bit Depth |
|-------------|-------|-------------|-----------|
| **sRGB** | Standard (≈35% visible) | Web, consumer displays | 8-bit |
| **Adobe RGB** | Wide (≈50% visible) | Print, photography | 8/16-bit |
| **ProPhoto RGB** | Very wide (≈90% visible) | Professional photography | 16-bit |
| **Display P3** | Wide (≈45% visible) | Apple devices, HDR | 10/16-bit |
| **Rec.2020** | Ultra-wide | HDR video, cinema | 10/12-bit |
| **ACES** | Scene-referred | VFX, cinema | 16/32-bit float |

### 6.3 Color Conversion Pipeline

```typescript
interface ColorManagement {
  documentProfile: ICCProfile;
  workingSpace: ICCProfile;
  proofProfile?: ICCProfile;
  
  // Conversion settings
  renderingIntent: RenderingIntent;
  blackPointCompensation: boolean;
  dither: boolean;
  
  // Transform caching for performance
  transformCache: Map<string, ColorTransform>;
}

// Conversion flow:
// Source → PCS (Lab/XYZ) → Destination
function convertColor(
  color: Color,
  sourceProfile: ICCProfile,
  destProfile: ICCProfile,
  intent: RenderingIntent
): Color {
  // 1. Source to PCS
  const pcsColor = sourceProfile.toPCS(color, intent);
  
  // 2. Chromatic adaptation if white points differ
  const adaptedPCS = chromaticAdaptation(
    pcsColor,
    sourceProfile.mediaWhitePoint,
    destProfile.mediaWhitePoint,
    'bradford'  // Adaptation method
  );
  
  // 3. PCS to destination
  return destProfile.fromPCS(adaptedPCS, intent);
}
```

### 6.4 Bit Depth Support

| Depth | Values per Channel | Dynamic Range | Use Case |
|-------|-------------------|---------------|----------|
| 8-bit | 256 | ~6 stops | Web, display |
| 16-bit | 65,536 | ~11 stops | Photography, editing |
| 32-bit float | Unlimited | Unlimited | HDR, compositing |

---

## 7. File Format Support

### 7.1 Native Document Formats

#### PSD (Photoshop Document)
```typescript
interface PSDFormat {
  maxDimension: 30000;           // Pixels
  maxFileSize: 2_000_000_000;    // 2GB
  
  supports: {
    layers: true;
    masks: true;
    alphaChannels: true;
    spotColors: true;
    clippingPaths: true;
    adjustmentLayers: true;
    smartObjects: true;
    vectorShapes: true;
    text: true;
    layerEffects: true;
    blendModes: 'all';
    bitDepth: [8, 16, 32];
    colorModes: ['Bitmap', 'Grayscale', 'RGB', 'CMYK', 'Lab', 'Multichannel'];
  };
}
```

#### PSB (Large Document Format)
- Extension of PSD for large files
- Max dimensions: 300,000 × 300,000 pixels
- Max file size: ~4 exabytes

### 7.2 TIFF (Tagged Image File Format)

```typescript
interface TIFFCapabilities {
  compression: [
    'none',
    'lzw',
    'zip',
    'jpeg',
    'packbits'
  ];
  
  bitDepth: [1, 8, 16, 32];
  colorSpaces: ['bilevel', 'grayscale', 'rgb', 'cmyk', 'lab'];
  
  // Photoshop-specific TIFF features
  layersSupport: true;           // Via private tags
  iccProfile: true;
  multiPage: true;
  tiling: true;
  bigTiff: true;                 // >4GB files
}
```

### 7.3 OpenEXR (High Dynamic Range)

```typescript
interface EXRFormat {
  // Color depth options
  bitDepth: {
    half: 16;                    // 16-bit float
    float: 32;                   // 32-bit float
    uint: 32;                    // 32-bit unsigned int
  };
  
  // Compression methods
  compression: [
    'none',
    'rle',                       // Run-length encoding
    'zip',                       // Scanline zip
    'zip16',                     // 16-scanline blocks
    'piz',                       // Wavelet, best for grain
    'pxr24',                     // Pixar 24-bit
    'b44',                       // Lossy, fixed ratio
    'b44a',                      // B44 + flat area compression
    'dwaa',                      // DreamWorks lossy
    'dwab'                       // DreamWorks lossy (256 scanlines)
  ];
  
  // Key features
  features: {
    arbitraryChannels: true;     // RGB, specular, normals, etc.
    multiView: true;             // Stereo left/right
    deepData: true;              // Deep compositing (EXR 2.0)
    multiPart: true;             // Multiple images in one file
    tiling: true;
    mipmaps: true;
  };
}
```

### 7.4 RAW Formats

| Format | Manufacturer | Based On | Documented |
|--------|--------------|----------|------------|
| **DNG** | Adobe | TIFF/EP | Yes (public spec) |
| **CR2/CR3** | Canon | TIFF (CR2), ISO BMF (CR3) | Reverse-engineered |
| **NEF** | Nikon | TIFF/EP | Reverse-engineered |
| **ARW** | Sony | TIFF | Reverse-engineered |
| **ORF** | Olympus | Custom | Reverse-engineered |
| **RAF** | Fujifilm | Custom | Reverse-engineered |
| **RW2** | Panasonic | Custom | Reverse-engineered |

#### RAW Processing Pipeline

```
┌─────────┐   ┌──────────┐   ┌───────────┐   ┌─────────┐
│ Decode  │──▶│ Demosaic │──▶│ Color     │──▶│ Develop │
│ Sensor  │   │ (Bayer)  │   │ Profile   │   │ Params  │
└─────────┘   └──────────┘   └───────────┘   └─────────┘
     │             │              │               │
  [14-bit]    [Interpolate    [Camera to     [Exposure,
   linear]     missing RGB]    working       curves,
                               space]        etc.]
```

---

## 8. Batch Processing Pipelines

### 8.1 Action/Macro System

```typescript
interface Action {
  id: string;
  name: string;
  steps: ActionStep[];
  
  // Execution options
  playbackOptions: {
    accelerated: boolean;        // Skip dialogs
    stepByStep: boolean;         // Pause between steps
    logResults: boolean;
  };
}

interface ActionStep {
  command: string;               // Operation identifier
  parameters: Record<string, any>;
  
  // Control flow
  conditional?: {
    check: 'hasSelection' | 'documentMode' | 'layerType' | 'custom';
    action: 'skip' | 'stop' | 'branch';
  };
}

// Batch execution
interface BatchProcessor {
  sourceFolder: string;
  destinationFolder: string;
  filePattern: string;           // e.g., "*.jpg"
  
  actions: Action[];
  
  // Output options
  saveOptions: {
    format: string;
    quality?: number;
    naming: NamingPattern;
    subfolder?: string;
  };
  
  // Error handling
  onError: 'stop' | 'skip' | 'log';
  
  execute(): Promise<BatchResult>;
}
```

### 8.2 Image Processor Architecture

```typescript
interface ImageProcessor {
  // Input
  sources: ImageSource[];
  
  // Processing pipeline
  pipeline: ProcessingStage[];
  
  // Output
  outputs: OutputConfig[];
  
  // Parallel execution
  concurrency: number;
  
  async process(): AsyncGenerator<ProcessingProgress>;
}

interface ProcessingStage {
  type: 'resize' | 'adjust' | 'filter' | 'action' | 'script';
  config: StageConfig;
  condition?: (image: ImageMetadata) => boolean;
}

interface OutputConfig {
  path: string | ((source: string) => string);
  format: ImageFormat;
  quality: number;
  metadata: MetadataConfig;
}
```

### 8.3 Droplet/Standalone Executables

```typescript
interface Droplet {
  // Encapsulated action + settings
  action: Action;
  batchSettings: BatchSettings;
  
  // Self-contained executable
  compile(): Executable;
  
  // Can process files by drag-and-drop
  processFiles(paths: string[]): void;
}
```

---

## 9. Plugin/Extension Architectures

### 9.1 Photoshop Plugin Types

| Type | Extension (Win) | Mac Code | Purpose |
|------|-----------------|----------|---------|
| **Filter** | .8bf | 8BFM | Image effects |
| **Import** | .8ba | 8BAM | Acquire from devices |
| **Export** | .8be | 8BEM | Save to formats |
| **File Format** | .8bi | 8BIF | Open/save formats |
| **Automation** | .8li | 8LIZ | Scripts, macros |
| **Color Picker** | .8bc | 8BCM | Custom color dialogs |
| **Extension** | .8bx | 8BXM | UI panels (CEP/UXP) |

### 9.2 Plugin API Structure

```c
// Photoshop Plugin SDK (simplified)
typedef struct {
    int16 version;
    
    // Host callbacks
    FilterRecord* filterRecord;
    
    // Plugin info
    PlugInInfo* plugInInfo;
    
    // Image access
    void* inData;
    void* outData;
    int32 inRowBytes;
    int32 outRowBytes;
    
    // Processing region
    Rect16 filterRect;
    int16 plane;
    
    // Progress
    void (*progressProc)(int32 done, int32 total);
    void (*abortProc)(void);
    
} FilterRecord;

// Entry point
DLLExport MACPASCAL void PluginMain(
    const int16 selector,
    FilterRecord* filterRecord,
    int32* data,
    int16* result
);
```

### 9.3 Modern Extension Architecture (CEP/UXP)

```typescript
// UXP Plugin Manifest (manifest.json)
interface UXPManifest {
  id: string;
  name: string;
  version: string;
  host: {
    app: 'PS' | 'AI' | 'ID';
    minVersion: string;
  };
  entrypoints: {
    type: 'panel' | 'command';
    id: string;
    label: string;
    script: string;
  }[];
}

// UXP API access
const { app, imaging } = require('photoshop');

// Document manipulation
const doc = app.activeDocument;
const layer = doc.activeLayers[0];

// Pixel access
const imageData = await imaging.getPixels({
  documentID: doc.id,
  layerID: layer.id,
  targetSize: { width: 100, height: 100 }
});
```

### 9.4 GIMP Plugin System

```python
# GIMP Python-Fu Plugin
from gimpfu import *

def my_plugin(image, drawable, param1, param2):
    """Plugin implementation"""
    gimp.progress_init("Processing...")
    
    # Get pixel region
    pixel_region = drawable.get_pixel_rgn(0, 0, 
        drawable.width, drawable.height, True, True)
    
    # Process pixels
    for y in range(drawable.height):
        for x in range(drawable.width):
            pixel = pixel_region[x, y]
            # Modify pixel...
            pixel_region[x, y] = new_pixel
        
        gimp.progress_update(float(y) / drawable.height)
    
    drawable.flush()
    drawable.merge_shadow(True)
    drawable.update(0, 0, drawable.width, drawable.height)

register(
    "my_plugin",                    # Name
    "Description",                   # Blurb
    "Help text",                     # Help
    "Author", "Copyright", "2024",   # Attribution
    "My Plugin",                     # Menu label
    "RGB*, GRAY*",                   # Image types
    [
        (PF_IMAGE, "image", "Input image", None),
        (PF_DRAWABLE, "drawable", "Input drawable", None),
        (PF_INT, "param1", "Parameter 1", 50),
        (PF_FLOAT, "param2", "Parameter 2", 1.0),
    ],
    [],
    my_plugin,
    menu="<Image>/Filters/Custom"
)

main()
```

---

## 10. AI-Powered Features

### 10.1 Content-Aware Technologies

#### Inpainting (Content-Aware Fill)

```typescript
interface InpaintingEngine {
  // Traditional approaches
  structural: {
    method: 'pde' | 'exemplar' | 'texture';
    propagateIsophotes: boolean;
  };
  
  // Deep learning approaches
  neural: {
    model: 'GAN' | 'diffusion' | 'transformer';
    contextSize: number;
    iterations: number;
  };
  
  async inpaint(
    image: ImageBuffer,
    mask: MaskBuffer,
    options: InpaintOptions
  ): Promise<ImageBuffer>;
}

interface InpaintOptions {
  structureAdaptation: number;    // 0-1
  colorAdaptation: number;        // 0-1
  rotation: boolean;              // Allow patch rotation
  mirror: boolean;                // Allow mirroring
  samplingArea: 'auto' | BoundingBox;
}
```

#### Seam Carving (Content-Aware Scale)

```typescript
interface SeamCarver {
  // Energy function determines importance
  energyFunction: 'gradient' | 'entropy' | 'saliency' | 'custom';
  
  // Protection/removal masks
  protectMask?: MaskBuffer;
  removeMask?: MaskBuffer;
  
  resize(
    image: ImageBuffer,
    newWidth: number,
    newHeight: number
  ): ImageBuffer;
  
  // Get seam visualization
  visualizeSeams(): ImageBuffer;
}
```

### 10.2 AI Upscaling / Super Resolution

```typescript
interface SuperResolution {
  // Model types
  model: 
    | 'ESRGAN'           // Enhanced SRGAN
    | 'Real-ESRGAN'      // Real-world images
    | 'BSRGAN'           // Blind SR
    | 'SwinIR'           // Transformer-based
    | 'Stable-Diffusion' // Diffusion-based
    | 'DLSS'             // NVIDIA deep learning
    | 'FSR';             // AMD FidelityFX
  
  scaleFactor: 2 | 4 | 8;
  
  // Enhancement options
  denoising: number;            // 0-1
  faceEnhancement: boolean;
  
  async upscale(image: ImageBuffer): Promise<ImageBuffer>;
}
```

### 10.3 Neural Style Transfer

```typescript
interface StyleTransfer {
  contentImage: ImageBuffer;
  styleImage: ImageBuffer;
  
  // Transfer parameters
  contentWeight: number;         // Preserve structure
  styleWeight: number;           // Apply style
  
  // Multi-scale options
  scales: number[];              // e.g., [256, 512, 1024]
  iterations: number;
  
  // Preserve colors option
  preserveColors: boolean;
  
  async transfer(): Promise<ImageBuffer>;
}
```

### 10.4 Generative AI Features

```typescript
interface GenerativeFill {
  // Text-to-image generation within selection
  prompt: string;
  negativePrompt?: string;
  
  // Context from surrounding area
  contextPadding: number;
  
  // Generation parameters
  guidanceScale: number;         // CFG scale
  steps: number;                 // Diffusion steps
  seed?: number;                 // Reproducibility
  
  // Multiple variations
  numVariations: number;
  
  async generate(
    image: ImageBuffer,
    mask: MaskBuffer
  ): Promise<ImageBuffer[]>;
}

interface GenerativeExpand {
  // Outpainting / image extension
  direction: 'all' | 'horizontal' | 'vertical';
  expandPixels: number;
  
  prompt?: string;               // Guide generation
  
  async expand(image: ImageBuffer): Promise<ImageBuffer>;
}
```

### 10.5 AI-Assisted Selection

```typescript
interface AISelection {
  // Subject detection
  selectSubject(): Promise<MaskBuffer>;
  
  // Sky detection
  selectSky(): Promise<MaskBuffer>;
  
  // Object-aware selection
  selectObject(point: Point2D): Promise<MaskBuffer>;
  
  // Semantic segmentation
  segmentAll(): Promise<SegmentationResult>;
  
  // Edge refinement
  refineEdge(
    mask: MaskBuffer,
    options: RefineOptions
  ): Promise<MaskBuffer>;
}

interface RefineOptions {
  radius: number;
  smooth: number;
  feather: number;
  contrast: number;
  shiftEdge: number;
  decontaminate: boolean;
}
```

### 10.6 AI Model Integration Architecture

```typescript
interface AIBackend {
  // Model loading
  loadModel(modelPath: string): Promise<Model>;
  
  // Execution backends
  backend: 'cpu' | 'cuda' | 'metal' | 'directml' | 'coreml';
  
  // Memory management
  maxMemoryGB: number;
  tileSize: number;              // For large images
  
  // Inference
  async infer(
    model: Model,
    inputs: Tensor[],
    options: InferenceOptions
  ): Promise<Tensor[]>;
}

interface Model {
  // Model metadata
  name: string;
  version: string;
  inputShape: number[];
  outputShape: number[];
  
  // Quantization
  precision: 'fp32' | 'fp16' | 'int8';
  
  // Warmup for consistent performance
  warmup(): Promise<void>;
}
```

---

## Appendix A: Reference Implementations

### Open Source Projects
- **GIMP** + **GEGL**: https://gitlab.gnome.org/GNOME/gimp
- **Krita**: https://invent.kde.org/graphics/krita
- **RawTherapee**: https://github.com/Beep6581/RawTherapee
- **darktable**: https://github.com/darktable-org/darktable
- **ImageMagick**: https://github.com/ImageMagick/ImageMagick

### File Format Specifications
- **PSD**: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
- **DNG**: https://helpx.adobe.com/photoshop/digital-negative.html
- **OpenEXR**: https://openexr.com/
- **ICC Profiles**: https://www.color.org/icc_specs2.xalter

### Color Science Resources
- **ICC Specification**: ISO 15076-1:2010
- **sRGB**: IEC 61966-2-1:1999
- **babl**: https://gegl.org/babl/

---

*Document Version: 1.0*
*Last Updated: 2026-01-28*
*Compiled from: Adobe Photoshop documentation, GIMP/GEGL source code, ICC specifications, Wikipedia technical articles, Affinity Photo documentation*
