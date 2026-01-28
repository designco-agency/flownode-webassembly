# FlowNode React Specification

> Complete specification for the FlowNode React codebase to enable WASM implementation parity.
> Generated from source analysis of `designco-node/` repository.

---

## Table of Contents

1. [Node Types](#node-types)
2. [Keyboard Shortcuts](#keyboard-shortcuts)
3. [Node Properties & Parameters](#node-properties--parameters)
4. [Connection Rules](#connection-rules)
5. [File Format](#file-format)
6. [Layout Constants](#layout-constants)

---

## Node Types

### Registered Node Types

The following node types are registered in `src/constants/editorConfig.ts`:

| Type Key | Component | Category | Description |
|----------|-----------|----------|-------------|
| `image` | ImageNode | Content | Image display and manipulation |
| `content` | ContentNode | Content | Universal content (images, videos, docs) |
| `text` | TextNode | Text | Text input with AI enhancement |
| `concat` | ConcatNode | Text | Concatenate two text inputs |
| `llm` | LLMNode | AI | Text generation via LLM |
| `omni` | OmniNode | AI | Image generation (multi-model) |
| `compare` | CompareNode | Utility | Side-by-side image comparison |
| `bucket` | ImageBucketNode | Content | Multi-image container |
| `postit` | PostItNode | Note | Sticky note |
| `splitter` | LineSplitterNode | Text | Split text by delimiter |
| `composition` | CompositionNode | Editor | Layer-based image composition |
| `upscaler` | UpscalerNode | AI | Image upscaling |
| `vector` | VectorNode | AI | Rasterize to SVG conversion |
| `group` | GroupNode | Utility | Group container |
| `folder` | FolderNode | Utility | Folder container |
| `rodin3d` | Rodin3DNode | AI | 3D model generation |
| `title` | TitleNode | UI | Title/label node |
| `mind-map` | MindMapNode | AI | Mind mapping with AI |
| `video` | VideoNode | AI | Video generation |
| `batch` | BatchNode | Utility | Batch input manager |
| `groupProxy` | GroupProxyNode | Utility | Group proxy node |
| `convertor` | ConvertorNode | Utility | Video/image format conversion |
| `router` | RouterNode | Utility | Signal splitter/router |
| `adjust` | ImageAdjustNode | Editor | Image color adjustments |
| `effects` | EffectsNode | Editor | Image effects (blur, grain, etc.) |

---

## Keyboard Shortcuts

### Node Creation Shortcuts

Single-key shortcuts (no modifiers) create nodes at cursor position:

| Key | Node Type | Description |
|-----|-----------|-------------|
| `T` | `text` | Text Node |
| `N` | `postit` | Post-It Note |
| `I` | `image` | Image Node |
| `B` | `bucket` | Image Bucket |
| `J` | `concat` | Concat (Join) |
| `S` | `splitter` | Line Splitter |
| `C` | `compare` | Compare Node |
| `F` | `composition` | Composition (Frame) |
| `O` | `omni` | Omni (Image Gen) |
| `L` | `llm` | LLM Node |
| `U` | `upscaler` | Upscaler |
| `V` | `vector` | Vector Node |
| `3` | `rodin3d` | 3D Generation |
| `H` | `title` | Title (Header) |
| `M` | `mind-map` | Mind Map |
| `K` | `content` | Content Node |
| `D` | `video` | Video (Director) |
| `Q` | `batch` | Batch (Queue) |
| `R` | `router` | Router |
| `A` | `adjust` | Image Adjust |
| `E` | `effects` | Effects |

### Editor Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + C` | Copy selected nodes |
| `Ctrl/Cmd + V` | Paste nodes |
| `Ctrl/Cmd + D` | Duplicate selected nodes |
| `Ctrl/Cmd + G` | Group selected nodes |
| `Ctrl/Cmd + Shift + G` | Ungroup selected |
| `Space` (hold) | Pan mode |
| `Shift + .` or `>` | Toggle Pro Mode |
| `Escape` | Close fullscreen/modals |
| `Z` (hold) | Zoom mode (in some nodes) |

### Shortcut Conditions

Shortcuts are ignored when:
- User is typing in `<input>`, `<textarea>`, or `contentEditable`
- Canvas is in fullscreen mode (`[data-canvas-fullscreen="true"]`)
- Modifier keys (Ctrl/Cmd/Alt) are held (except for specific combos)

---

## Node Properties & Parameters

### TextNode

**Data Interface: `TextNodeData`**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `text` | `string` | `''` | Text content |
| `lockText` | `boolean` | `false` | Prevent auto-sync from upstream |

**Handles:**
- **Input:** `text-in` (left) - Text
- **Output:** `text-out` (right) - Text

**Features:**
- AI text enhancement (Standard/Advanced)
- `@variable` autocomplete for connected images
- JSON output mode toggle

---

### LLMNode

**Data Interface: `LLMNodeData`**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `model` | `string` | `'gpt-5.2'` | `gemini-3-pro-preview`, `gemini-3-flash-preview`, `gpt-5.2`, `gemini-1.5-pro`, `gemini-1.5-flash`, `gemini-1.0-pro` |
| `dontCompress` | `boolean` | `false` | Skip image compression |
| `internalImage` | `string` | `null` | Manually added image |
| `prompt` | `string` | `''` | Prompt text |
| `text` | `string` | `''` | Output text |

**Handles:**
- **Input:** `prompt-in` (left) - Prompt text
- **Input:** `content-in` (left) - Images/documents (up to 10)
- **Output:** `llm-out` (right) - Generated text

---

### OmniNode (Image Generation)

**Data Interface: `OmniNodeData`**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `model` | `string` | `'gemini-3-pro-image-preview'` | Model selection |
| `resolution` | `string` | `'1K'` | Output resolution |
| `aspectRatio` | `string` | `'1:1'` | Aspect ratio |
| `matchInputRatio` | `boolean` | `true` | Match input image ratio |
| `runCount` | `number` | `1` | Number of generations |
| `lockSeed` | `boolean` | `false` | Lock seed for reproducibility |
| `lastSeed` | `number|string` | - | Last used seed |
| `image` | `string` | - | Result image URL |
| `imageOrder` | `string[]` | - | Order of input images |
| `promptCount` | `number` | - | Number of prompt slots |
| `promptOrder` | `string[]` | - | Order of prompts |

**Ideogram-Specific:**
| Property | Type | Default | Options |
|----------|------|---------|---------|
| `styleType` | `string` | `'None'` | Style type |
| `stylePreset` | `string` | `'None'` | Style preset |
| `magicPromptOption` | `string` | `'Auto'` | Magic prompt mode |
| `isMultiMode` | `boolean` | `false` | Multi-image mode |

**Qwen Multiple Angles:**
| Property | Type | Default | Range |
|----------|------|---------|-------|
| `verticalAngle` | `number` | `0` | Camera vertical angle |
| `rotateRightLeft` | `number` | `0` | Rotation |
| `moveForward` | `number` | `0` | Camera distance |
| `wideAngleLens` | `boolean` | `false` | Wide angle toggle |
| `loraScale` | `number` | `1.25` | LoRA scale |
| `numLayers` | `number` | `4` | For layered output |

**Handles:**
- **Input:** `prompts-in` (left) - Text prompts (multiple)
- **Input:** `images-in` (left) - Reference images (multiple)
- **Output:** `content-out` (right) - Single output
- **Output:** `multi-content-out` (right) - Multi-output

---

### VideoNode

**Data Interface: `VideoNodeData`**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `prompt` | `string` | `''` | Video prompt |
| `negativePrompt` | `string` | `''` | Negative prompt |
| `duration` | `number` | `4` | 4, 5, 6, 8, 10 seconds |
| `aspectRatio` | `string` | `'16:9'` | `16:9`, `9:16`, `1:1` |
| `model` | `string` | `'veo-3.1-gemini'` | Model selection |
| `resolution` | `string` | `'720p'` | `720p`, `1080p` |
| `enableAudio` | `boolean` | `false` | Native audio |
| `seed` | `number` | - | Random seed |
| `imageRoles` | `Record<string, string>` | - | Role per image: `first`, `last`, `reference`, `motion`, `audio`, `video` |
| `motionControlMode` | `string` | `'std'` | `std`, `pro` |
| `characterOrientation` | `string` | `'image'` | `image`, `video` |
| `keepOriginalSound` | `boolean` | `false` | Keep original audio |

**Handles:**
- **Input:** `prompts-in` (left) - Prompts
- **Input:** `images-in` (left) - Reference images
- **Output:** `content-out` (right) - Video output

---

### ImageNode

**Data Interface: `ImageNodeData`**

| Property | Type | Description |
|----------|------|-------------|
| `image` | `string` | Current image URL |
| `thumbnail` | `string` | Thumbnail URL |
| `history` | `string[]` | Image history (max 10) |
| `isGenerating` | `boolean` | Generation in progress |
| `generatingIndex` | `number` | Current generation index |
| `generatingTotal` | `number` | Total generations |
| `generationError` | `boolean` | Error state |
| `meta` | `object` | Metadata (file_size, resolution) |
| `isExpanded` | `boolean` | Expanded view state |
| `lastSourceImage` | `string` | Tracks upstream sync |

**Handles:**
- **Input:** `content-in` (left) - Image input
- **Output:** `content-out` (right) - Image output

**Features:**
- Background removal
- Cropping with rotation/zoom
- Eraser tool
- History navigation (← →)
- Favorite toggle
- Fullscreen lightbox

---

### ImageAdjustNode

**Adjustment Settings:**

| Property | Type | Default | Range |
|----------|------|---------|-------|
| `brightness` | `number` | `0` | -100 to 100 |
| `contrast` | `number` | `0` | -100 to 100 |
| `saturation` | `number` | `0` | -100 to 100 |
| `exposure` | `number` | `0` | -100 to 100 |
| `highlights` | `number` | `0` | -100 to 100 |
| `shadows` | `number` | `0` | -100 to 100 |
| `temperature` | `number` | `0` | -100 to 100 (cool to warm) |
| `tint` | `number` | `0` | -100 to 100 (green to magenta) |
| `vibrance` | `number` | `0` | -100 to 100 |
| `gamma` | `number` | `0` | -100 to 100 (maps to 0.1-3.0) |

**Color Grading (per wheel: lift, gamma, gain, offset):**
| Property | Type | Default | Range |
|----------|------|---------|-------|
| `x` | `number` | `0` | -1 to 1 |
| `y` | `number` | `0` | -1 to 1 |
| `luminance` | `number` | `0` | -100 to 100 |

**Additional:**
| Property | Type | Default | Range |
|----------|------|---------|-------|
| `colorBoost` | `number` | `0` | -100 to 100 |
| `hueRotation` | `number` | `0` | -180 to 180 |
| `luminanceMix` | `number` | `100` | 0 to 100 |

**Curves:**
- RGB curves with control points

**Handles:**
- **Input:** `content-in` (left)
- **Output:** `content-out` (right)

---

### EffectsNode

**Effect Settings:**

| Property | Type | Default | Range | Description |
|----------|------|---------|-------|-------------|
| `gaussianBlur` | `number` | `0` | 0-100 | Gaussian blur |
| `directionalBlur` | `number` | `0` | 0-100 | Motion blur |
| `directionalBlurAngle` | `number` | `0` | 0-360 | Blur angle |
| `progressiveBlur` | `number` | `0` | 0-100 | Gradient blur |
| `progressiveBlurDirection` | `string` | `'bottom'` | `top`, `bottom`, `left`, `right` | Gradient direction |
| `progressiveBlurFalloff` | `number` | `50` | 0-100 | Transition smoothness |
| `glassBlinds` | `number` | `0` | 0-100 | Glass blinds effect |
| `glassBlindsFrequency` | `number` | `10` | 1-50 | Number of blinds |
| `glassBlindsAngle` | `number` | `0` | 0-360 | Blinds angle |
| `glassBlindsPhase` | `number` | `0` | 0-100 | Phase offset |
| `grain` | `number` | `0` | 0-100 | Film grain |
| `grainSize` | `number` | `2` | 1-10 | Grain size |
| `grainMonochrome` | `boolean` | `true` | - | B&W or color grain |
| `grainSeed` | `number` | `0` | - | Random seed |
| `sharpen` | `number` | `0` | 0-100 | Sharpening |
| `vignette` | `number` | `0` | 0-100 | Vignette intensity |
| `vignetteRoundness` | `number` | `50` | 0-100 | Shape |
| `vignetteSmoothness` | `number` | `50` | 0-100 | Edge softness |

**Handles:**
- **Input:** `content-in` (left)
- **Output:** `content-out` (right)

---

### UpscalerNode

**Models Available:**

| Model ID | Name | Scale Options |
|----------|------|---------------|
| `freepik-precision-v2` | Magnific Precision V2 Ultra | 2, 4, 8, 16 |
| `crystal-upscaler` | Crystal Upscaler | 2, 4 |
| `crystal-video-upscaler` | Crystal Video Upscaler | 1-4 |
| `google-upscaler` | Google Upscaler | x2, x4 |
| `seed-vr2` | SeedVR2 (Restoration) | - |

**Freepik Settings:**
| Property | Type | Default | Options/Range |
|----------|------|---------|---------------|
| `scale_factor` | `number` | `2` | 2, 4, 8, 16 |
| `flavor` | `string` | `'sublime'` | `sublime`, `photo`, `photo_denoiser` |
| `sharpen` | `number` | `7` | 0-100 |
| `smart_grain` | `number` | `7` | 0-100 |
| `ultra_detail` | `number` | `30` | 0-100 |

**Crystal Upscaler Settings:**
| Property | Type | Default | Range |
|----------|------|---------|-------|
| `scale` | `number` | `2` | 2, 4 |
| `dynamic` | `number` | `6` | 1-10 |
| `creativity` | `number` | `0.3` | 0-0.5 |
| `resemblance` | `number` | `0.6` | 0-1 |
| `output_format` | `string` | `'png'` | `png`, `jpeg`, `webp` |

**Handles:**
- **Input:** `content-in` (left)
- **Output:** `content-out` (right)

---

### Rodin3DNode

**Data Interface:**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `selectedModel` | `string` | `'rodin'` | `rodin`, `tripo` |
| `prompt` | `string` | `''` | Text prompt |
| `modelUrl` | `string` | - | Output 3D model URL |

**Rodin Settings:**
| Property | Type | Default | Options |
|----------|------|---------|---------|
| `condition_mode` | `string` | `'concat'` | `fuse`, `concat` |
| `geometry_file_format` | `string` | `'glb'` | `glb`, `usdz`, `fbx`, `obj`, `stl` |
| `material` | `string` | `'PBR'` | `PBR`, `Shaded` |
| `quality` | `string` | `'medium'` | `high`, `medium`, `low`, `extra-low` |
| `tier` | `string` | `'Regular'` | `Regular`, `Sketch` |
| `use_hyper` | `boolean` | `false` | - |
| `TAPose` | `boolean` | `false` | - |

**Tripo Settings:**
| Property | Type | Default | Options |
|----------|------|---------|---------|
| `tripo_texture` | `string` | `'standard'` | `no`, `standard`, `HD` |
| `tripo_pbr` | `boolean` | `true` | - |
| `tripo_face_limit` | `number` | - | Face count limit |
| `tripo_auto_size` | `boolean` | `false` | - |
| `tripo_quad` | `boolean` | `false` | Quad topology |
| `tripo_texture_alignment` | `string` | `'original_image'` | `original_image`, `geometry` |
| `tripo_orientation` | `string` | `'default'` | `default`, `align_image` |

**Handles (Rodin):**
- **Input:** `prompt-in` (left) - Text prompt
- **Input:** `images-in` (left) - Reference images (multiple)
- **Output:** `content-out` (right) - 3D model

**Handles (Tripo):**
- **Input:** `front-in`, `left-in`, `back-in`, `right-in` - Directional images

---

### BatchNode

**Data Interface: `BatchNodeData`**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `inputCount` | `number` | `5` | Number of input slots |
| `batchOutput` | `string[]` | `[]` | Collected content URLs |
| `batchType` | `string` | - | `text`, `image`, `mixed` |

**Handles:**
- **Input:** `in-0` through `in-N` (left) - Dynamic inputs
- **Output:** `batch-out` (right) - Batch output

---

### RouterNode

**Data Interface: `RouterNodeData`**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `outputCount` | `number` | `5` | Number of outputs |
| `image` | `string` | `null` | Routed content |
| `text` | `string` | `null` | Routed text |
| `routedType` | `string` | `'unknown'` | `content`, `text`, `unknown` |

**Handles:**
- **Input:** `router-in` (left) - Any input
- **Output:** `out-0` through `out-N` (right) - Dynamic outputs

---

### CompareNode

**Features:**
- Before/After slider comparison
- Fullscreen mode
- Expandable on canvas

**Handles:**
- **Input:** `content-1-in` (left) - Image 1
- **Input:** `content-2-in` (left) - Image 2

---

### ConcatNode

**Data Interface:**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `separator` | `string` | `'New Line'` | `New Line`, `Space`, `Comma` |
| `text` | `string` | - | Output text |

**Handles:**
- **Input:** `text-1` (left) - First text
- **Input:** `text-2` (left) - Second text
- **Output:** `concat-out` (right) - Combined text

---

### LineSplitterNode

**Data Interface:**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `separator` | `string` | `'New Line'` | `New Line`, `Comma`, `Period`, `Semicolon` |
| `outputCount` | `number` | `3` | Number of output lines |
| `lines` | `string[]` | - | Split output |

**Handles:**
- **Input:** `text-in` (left) - Input text
- **Output:** `line-0` through `line-N` (right) - Individual lines

---

### ConvertorNode

**Data Interface: `ConvertorNodeData`**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `format` | `string` | `'webm'` | `webm`, `mp4` |
| `quality` | `string` | `'medium'` | `high`, `medium`, `low` |
| `status` | `string` | `'idle'` | `idle`, `converting`, `done`, `error` |
| `progress` | `number` | `0` | 0-100 |
| `resultUrl` | `string` | - | Output URL |
| `targetDuration` | `number` | - | For image-to-video |

**Handles:**
- **Input:** `content-in` (left) - Image or video
- **Output:** `content-out` (right) - Converted output

---

### TitleNode

**Data Interface:**

| Property | Type | Default | Options |
|----------|------|---------|---------|
| `text` | `string` | `'Title'` | Title text |
| `size` | `string` | `'normal'` | `normal`, `big`, `super` |

**Size Config:**
| Size | Font Size | Font Weight |
|------|-----------|-------------|
| `normal` | 16px | 500 |
| `big` | 24px | 600 |
| `super` | 36px | 700 |

**Handles:**
- **Input:** `lock-target` (left) - Lock to another node

---

### PostItNode

**Data Interface:**

| Property | Type | Default |
|----------|------|---------|
| `text` | `string` | `''` |
| `isOff` | `boolean` | `false` |

**Handles:**
- **Input:** `text-in` (left)
- **Output:** `text-out` (right)

**Styling:** Yellow (#fff740) background, Comic Sans font

---

### CompositionNode

**Data Interface: `CompositionNodeData`**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `layers` | `Layer[]` | `[]` | Layer stack |
| `image` | `string` | - | Rendered output |
| `compositionX` | `number` | `100` | Canvas X offset |
| `compositionY` | `number` | `100` | Canvas Y offset |
| `compositionWidth` | `number` | `1200` | Canvas width |
| `compositionHeight` | `number` | `900` | Canvas height |
| `compositionFill` | `string` | `'#ffffff'` | Background color |

**Layer Types:**
- `ImageLayer` - Raster images
- `DrawingLayer` - Freehand drawings
- `ShapeLayer` - Rectangles, ellipses
- `GroupLayer` - Layer groups
- `TextLayer` - Text elements

**Tool Settings:**
| Property | Type | Default |
|----------|------|---------|
| `strokeColor` | `string` | `'#00ff9d'` |
| `strokeWidth` | `number` | `3` |
| `eraserSize` | `number` | `20` |
| `fillColor` | `string` | `'transparent'` |
| `textColor` | `string` | `'#ffffff'` |
| `fontFamily` | `string` | `'Commissioner'` |
| `fontWeight` | `number` | `400` |
| `fontSize` | `number` | `24` |

**Handles:**
- **Input:** `content-in`, `content-in-2` through `content-in-10` (left) - Images
- **Output:** `content-out` (right) - Rendered composition

---

### GroupNode

Container node for grouping other nodes.

---

### VectorNode

**Models:** `recraft-vectorize` (Recraft Vectorizer)

**Handles:**
- **Input:** `content-in` (left) - Raster image
- **Output:** `content-out` (right) - SVG output

---

### ContentNode

Universal content node supporting:
- Images
- Videos
- Audio
- Documents

**Content Types:** `'image' | 'video' | 'audio' | 'document' | 'unknown'`

**Handles:**
- **Input:** `content-in` (left)
- **Output:** `content-out` (right)

---

## Connection Rules

### Handle Types

| Type | Description | Color Class |
|------|-------------|-------------|
| `content` | Images, videos, any visual | `.react-flow__handle-blue` |
| `text` | Text/prompts | Default |
| `batch` | Batch outputs | `.react-flow__handle-orange` |
| `router` | Router signals | `.react-flow__handle-cyan` |

### Input Handle Mapping

Source content automatically connects to appropriate target handle:

**For `image`/`content` sources:**
| Target Node | Handle ID |
|-------------|-----------|
| upscaler | `content-in` |
| bucket | `content-in` |
| composition | `content-in` |
| image | `content-in` |
| omni | `content-in` |
| llm | `content-in` |
| compare | `content-1-in` |
| vector | `content-in` |
| rodin3d | `content-in` |
| video | `images-in` |
| batch | `in-0` |
| convertor | `content-in` |
| router | `router-in` |
| adjust | `content-in` |
| effects | `content-in` |

**For `text` sources:**
| Target Node | Handle ID |
|-------------|-----------|
| llm | `prompt-in` |
| omni | `prompts-in` |
| text | `text-in` |
| splitter | `text-in` |
| concat | `text-1` |
| postit | `text-in` |
| rodin3d | `prompt-in` |
| video | `prompts-in` |
| batch | `in-0` |
| router | `router-in` |

### Output Handle Mapping

**For `content` outputs:**
| Source Node | Handle ID |
|-------------|-----------|
| image | `content-out` |
| content | `content-out` |
| upscaler | `content-out` |
| omni | `content-out` |
| composition | `content-out` |
| bucket | `content-out-0` |
| vector | `content-out` |
| rodin3d | `content-out` |
| video | `content-out` |
| batch | `batch-out` |
| convertor | `content-out` |
| router | `out-0` |
| adjust | `content-out` |
| effects | `content-out` |

**For `text` outputs:**
| Source Node | Handle ID |
|-------------|-----------|
| llm | `llm-out` |
| text | `text-out` |
| concat | `concat-out` |
| splitter | `line-0` |
| batch | `batch-out` |
| router | `out-0` |

---

## File Format

### Workflow JSON Structure

```typescript
interface Workflow {
  id: string;                    // UUID
  name: string;                  // Display name
  nodes: Node[];                 // React Flow nodes
  edges: Edge[];                 // React Flow edges
  gallery?: GalleryItem[];       // Saved assets
  viewport?: {                   // Camera position
    x: number;
    y: number;
    zoom: number;
  };
  thumbnail?: string | null;     // Preview image URL
  created_at: string;            // ISO timestamp
  updated_at: string;            // ISO timestamp
  last_opened_at: string;        // ISO timestamp
  user_email?: string;           // Owner
  is_public: boolean;            // Visibility
  folder_id?: string | null;     // Folder reference
}
```

### Node Structure

```typescript
interface Node {
  id: string;                    // Unique ID (e.g., "image-1704067200000-a1b2c")
  type: string;                  // Node type key
  position: {
    x: number;
    y: number;
  };
  data: Record<string, unknown>; // Node-specific data
  style?: {
    width?: number;
    height?: number;
  };
  selected?: boolean;
  parentId?: string;             // For grouped nodes
  measured?: {
    width: number;
    height: number;
  };
}
```

### Edge Structure

```typescript
interface Edge {
  id: string;                    // e.g., "e-source-handle-target-handle-timestamp"
  source: string;                // Source node ID
  target: string;                // Target node ID
  sourceHandle?: string;         // Output handle ID
  targetHandle?: string;         // Input handle ID
  type?: string;                 // Edge type (default: "deletable")
  style?: {
    stroke?: string;
    strokeWidth?: number;
  };
}
```

### Gallery Item Structure

```typescript
interface GalleryItem {
  url: string;                   // Asset URL
  timestamp: Date;               // Creation time
  model?: string;                // Generation model
  prompt?: string;               // Used prompt
  sourceNodeId?: string;         // Origin node
  workflowId?: string;           // Workflow reference
  workflowName?: string;         // Workflow name
  duration?: number;             // For videos
  fileFormat?: string;           // For 3D models
}
```

### Node ID Generation

```typescript
const generateNodeId = (type: string) => 
  `${type}-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`;
```

### Edge ID Generation

```typescript
const generateEdgeId = (source: string, sourceHandle: string, target: string, targetHandle: string) =>
  `e-${source}-${sourceHandle}-${target}-${targetHandle}-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`;
```

---

## Layout Constants

### Handle Dimensions

| Constant | Value | Description |
|----------|-------|-------------|
| `HANDLE_SIZE_PX` | `24` | Handle circle size |
| `HANDLE_GAP_PX` | `20` | Gap from node edge |
| `HANDLE_OFFSET_PX` | `-44` | Handle position offset |
| `NODE_PADDING_PX` | `16` | Node internal padding |
| `HANDLE_OFFSET_NESTED_PX` | `-60` | Nested handle offset |

### Node Dimensions

| Node Type | Min Width | Min Height |
|-----------|-----------|------------|
| DEFAULT | 200 | 100 |
| TEXT | 320 | 320 |
| LLM | 320 | 420 |
| OMNI | 490 | 400 |
| COMPARE | 600 | 600 |
| IMAGE | 300 | 300 |
| RODIN | 340 | 500 |
| COMPOSITION | 300 | 200 |
| UPSCALER | 320 | 400 |
| VECTOR | 320 | 400 |
| VIDEO | 400 | 500 |

### Initial Node Dimensions (for creation)

| Type | Width | Height |
|------|-------|--------|
| image | 200 | 200 |
| content | 200 | 200 |
| text | 280 | 320 |
| llm | 278 | 300 |
| omni | 490 | 650 |
| compare | 600 | 600 |
| bucket | 350 | 400 |
| postit | 200 | 200 |
| composition | 500 | 400 |
| upscaler | 200 | 200 |
| vector | 320 | 400 |
| rodin3d | 340 | 580 |
| mind-map | 300 | 300 |
| folder | 300 | 200 |
| video | 400 | 500 |
| batch | 220 | 250 |
| groupProxy | 32 | 32 |
| convertor | 400 | 400 |
| router | 120 | 200 |
| adjust | 280 | 420 |
| effects | 280 | 380 |

### Z-Index Layers

| Layer | Value |
|-------|-------|
| DEFAULT | 0 |
| HANDLE | 1 |
| OVERLAY_CONTROLS | 10 |
| POWER_TOGGLE | 50 |

---

## Initial Node Data

Default data when creating new nodes:

```typescript
const getNodeInitialData = (type: string) => {
  switch (type) {
    case 'text': return { text: 'New Text Node' };
    case 'postit': return { text: '' };
    case 'omni': return { prompt: '' };
    case 'llm': return { prompt: '' };
    case 'rodin3d': return { prompt: '' };
    case 'title': return { text: 'Title', size: 'normal' };
    case 'mind-map': return { text: '', isRoot: true, isAiNode: false };
    case 'content': return { contentType: 'unknown' };
    case 'folder': return { label: 'New Folder', isOpen: true, expandedWidth: 300, expandedHeight: 200 };
    case 'video': return { prompt: '', duration: 4, aspectRatio: '16:9', model: 'veo-3.1-gemini', enableAudio: false };
    case 'batch': return { inputCount: 5, batchOutput: [] };
    case 'convertor': return { format: 'webm', quality: 'medium', status: 'idle' };
    case 'router': return { outputCount: 5, image: null, text: null, routedType: 'unknown' };
    default: return { label: type };
  }
};
```

---

## Edge Configuration

### Default Edge Options

```typescript
const defaultEdgeOptions = {
  type: 'deletable',
  animated: false,
};
```

### Edge Types

| Type | Component |
|------|-----------|
| `deletable` | PulseEdge |

---

## Common Node Data Properties

All nodes inherit from `BaseNodeData`:

```typescript
interface BaseNodeData extends Record<string, unknown> {
  label?: string;
  isOff?: boolean;              // Power toggle state
  loading?: boolean;            // Processing indicator
  lastRunDuration?: number;     // Execution time
  estimatedCost?: number;       // Cost estimate
}
```

---

## Transient States (Cleaned on Save)

These properties are removed when saving workflows:

- `loading`
- `isGenerating`
- `isProcessing`
- `generatingIndex`
- `generatingTotal`
- `loadingMessages`
- `loadingMessageOffset`
- `generationError`

---

*Document generated from FlowNode React source analysis.*
*Last updated: Based on source files from designco-node repository.*
