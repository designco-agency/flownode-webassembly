# FlowNode WASM Development Plan

> Rebuilding the WASM canvas to match FlowNode React exactly.
> Last updated: 2026-01-28

---

## 🎯 Vision

**FlowNode WASM** = GPU-accelerated canvas that mirrors the React app:
1. **Generate** → AI image/video generation (API calls)
2. **Edit** → Adjust + Effects nodes (WASM/GPU processing)
3. **Save** → Cloud sync via Supabase

---

## 📊 Current State vs Target

| Feature | Current WASM | Target (React Parity) |
|---------|--------------|----------------------|
| Node types | 14 (image processing) | 26 (full FlowNode) |
| Adjust node | Basic B/C only | 10 sliders + color wheels + curves |
| Effects node | Basic blur | 17 parameters (blur, grain, vignette, etc.) |
| Keyboard shortcuts | R/Delete/Esc | 22 node shortcuts + editor commands |
| Cloud sync | ❌ None | ✅ Supabase real-time |
| File format | Custom | React Flow compatible |

---

## 🗺️ Development Phases

### Phase 1: Core Editing Nodes ⬅️ IN PROGRESS
Match the `adjust` and `effects` nodes exactly.

#### 1.1 ImageAdjustNode ✅ BASIC COMPLETE
- [x] Rename `BrightnessContrast` → `adjust`
- [x] Add all 10 basic sliders:
  - [x] brightness (-100 to 100)
  - [x] contrast (-100 to 100)
  - [x] saturation (-100 to 100)
  - [x] exposure (-100 to 100)
  - [x] highlights (-100 to 100)
  - [x] shadows (-100 to 100)
  - [x] temperature (-100 to 100)
  - [x] tint (-100 to 100)
  - [x] vibrance (-100 to 100)
  - [x] gamma (-100 to 100 → maps to 0.1-3.0)
- [ ] Color grading wheels:
  - [ ] Lift (shadows)
  - [ ] Gamma (midtones)
  - [ ] Gain (highlights)
  - [ ] Offset (master)
- [ ] RGB curves editor
- [ ] Additional controls:
  - [ ] colorBoost (-100 to 100)
  - [ ] hueRotation (-180 to 180)
  - [ ] luminanceMix (0 to 100)

#### 1.2 EffectsNode ✅ UI COMPLETE (Processing partial)
- [x] Rename `Blur` → `effects`
- [x] Blur effects:
  - [x] gaussianBlur (0-100) ✅ Processing done
  - [x] directionalBlur (0-100) - UI only
  - [x] directionalBlurAngle (0-360) - UI only
  - [x] progressiveBlur (0-100) - UI only
  - [x] progressiveBlurDirection (top/bottom/left/right)
  - [x] progressiveBlurFalloff (0-100)
- [x] Glass blinds:
  - [x] glassBlinds (0-100) - UI only
  - [x] glassBlindsFrequency (1-50)
  - [x] glassBlindsAngle (0-360)
  - [x] glassBlindsPhase (0-100)
- [x] Grain:
  - [x] grain (0-100) ✅ Processing done
  - [x] grainSize (1-10)
  - [x] grainMonochrome (bool)
  - [x] grainSeed (number)
- [x] Other:
  - [x] sharpen (0-100) ✅ Processing done
  - [x] vignette (0-100) ✅ Processing done
  - [x] vignetteRoundness (0-100)
  - [x] vignetteSmoothness (0-100)

### Phase 2: Node Types
Add remaining content/utility nodes.

- [ ] `image` - Image display with history
- [ ] `content` - Universal content node
- [ ] `bucket` - Multi-image container
- [ ] `text` - Text input
- [ ] `concat` - Text concatenation
- [ ] `splitter` - Text splitting
- [ ] `compare` - Side-by-side comparison
- [ ] `composition` - Layer-based editor
- [ ] `router` - Signal routing
- [ ] `batch` - Batch processing
- [ ] `title` - Labels
- [ ] `postit` - Sticky notes
- [ ] `group` / `folder` - Organization

### Phase 3: AI Generation Nodes
API integration for generation.

- [ ] `omni` - Multi-model image gen
- [ ] `llm` - Text generation
- [ ] `video` - Video generation
- [ ] `upscaler` - Image upscaling
- [ ] `vector` - SVG conversion
- [ ] `rodin3d` - 3D generation
- [ ] `mind-map` - AI mind mapping

### Phase 4: Keyboard Shortcuts
Match React shortcuts exactly.

| Key | Node | Status |
|-----|------|--------|
| `T` | text | ⬜ |
| `N` | postit | ⬜ |
| `I` | image | ⬜ |
| `B` | bucket | ⬜ |
| `J` | concat | ⬜ |
| `S` | splitter | ⬜ |
| `C` | compare | ⬜ |
| `F` | composition | ⬜ |
| `O` | omni | ⬜ |
| `L` | llm | ⬜ |
| `U` | upscaler | ⬜ |
| `V` | vector | ⬜ |
| `3` | rodin3d | ⬜ |
| `H` | title | ⬜ |
| `M` | mind-map | ⬜ |
| `K` | content | ⬜ |
| `D` | video | ⬜ |
| `Q` | batch | ⬜ |
| `R` | router | ⬜ |
| `A` | adjust | ⬜ |
| `E` | effects | ⬜ |

Editor shortcuts:
- [ ] `Ctrl/Cmd + C` - Copy
- [ ] `Ctrl/Cmd + V` - Paste
- [ ] `Ctrl/Cmd + D` - Duplicate
- [ ] `Ctrl/Cmd + G` - Group
- [ ] `Ctrl/Cmd + Shift + G` - Ungroup
- [ ] `Space` (hold) - Pan mode
- [ ] `Escape` - Close/cancel

### Phase 5: Cloud Sync
Supabase integration.

- [ ] Authentication (same as React app)
- [ ] Load workflows from cloud
- [ ] Save workflows to cloud
- [ ] Real-time sync
- [ ] Collaboration (future)

### Phase 6: File Format Compatibility
Full React Flow format support.

- [ ] Import React Flow JSON
- [ ] Export React Flow JSON
- [ ] Preserve unknown fields (round-trip)
- [ ] Handle ID convention (`input-0`, `output-0`)
- [ ] Node ID format (`{type}-{timestamp}-{random}`)

---

## 📁 File Structure

```
flownode-webassembly/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Main UI, panels, menus
│   ├── graph.rs          # Node graph rendering
│   ├── nodes.rs          # Node type definitions
│   ├── executor.rs       # Graph execution engine
│   ├── image_data.rs     # Image handling
│   ├── gpu.rs            # WebGPU processing
│   ├── compat.rs         # React Flow format conversion
│   ├── ui_components.rs  # Reusable UI widgets
│   ├── cloud.rs          # Supabase integration (TODO)
│   └── shaders/          # WGSL compute shaders (TODO)
│       ├── adjust.wgsl
│       ├── blur.wgsl
│       ├── grain.wgsl
│       └── vignette.wgsl
├── docs/
│   ├── REACT_SPEC.md     # Full React specification
│   ├── DEVELOPMENT_PLAN.md # This file
│   └── FORMAT_COMPATIBILITY.md
└── assets/
```

---

## 🔧 Technical Notes

### Parameter Mapping
React uses -100 to 100 for most sliders, mapped internally:
- `gamma`: -100→100 maps to 0.1→3.0
- `temperature`: -100→100 maps to cool→warm color shift
- etc.

### Color Wheels
Each wheel (lift/gamma/gain/offset) has:
- `x`: -1 to 1 (color hue on wheel)
- `y`: -1 to 1 (color hue on wheel)
- `luminance`: -100 to 100 (brightness)

### GPU Shaders
All processing should be GPU-accelerated via WGSL compute shaders.
Fallback to CPU for browsers without WebGPU.

---

## 📝 Session Log

### 2026-01-28
- ✅ Built initial MVP (14 node types, basic processing)
- ✅ Deployed to Vercel
- ✅ Sub-agent audited React codebase
- ✅ Created REACT_SPEC.md (27KB)
- ✅ Created this development plan
- ✅ **Major Refactor:** Restructured to match React spec exactly
  - 26 node types (all React types)
  - Adjust node: 10 sliders + color boost/hue/luminance
  - Effects node: All 17 parameters in UI
  - All keyboard shortcuts matching React
  - React Flow JSON format compatibility
- 🔄 Next: Color wheels, remaining effect processing, cloud sync

---

## 🔗 Links

- **Live:** https://flownode-webassembly.vercel.app
- **Repo:** https://github.com/designco-agency/flownode-webassembly
- **React app:** https://flownode.io
- **React repo:** ~/Documents/Github-Repositories/designco-node/
