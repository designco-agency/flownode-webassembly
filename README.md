# FlowNode Canvas 🎨

**The "Game Engine" approach to node-based image editing.**

This is a complete rebuild of FlowNode.io using a canvas-only architecture. No React, no HTML buttons, no CSS. The entire application is rendered as pixels on a single `<canvas>` by WebAssembly.

## Architecture

| Component | Technology | Purpose |
|-----------|------------|---------|
| **GUI Framework** | egui | Immediate-mode GUI library for Rust. Blazing fast. |
| **Node Graph** | egui_node_graph | Rust equivalent of React Flow |
| **Graphics** | glow (OpenGL) | Cross-platform rendering |
| **Deployment** | Trunk | Bundles Rust → .wasm + index.html |

## Why Canvas-Only?

### ✅ The Good

- **Incredible Performance**: Feels like a native desktop app (because it basically is one)
- **Code Sharing**: Same codebase compiles to Windows/Mac .exe AND Web .wasm
- **Unified Logic**: 100% Rust, no JS ↔ Wasm bridge complexity
- **Pixel Perfect Control**: No fighting with CSS z-index or overflow

### ⚠️ The Trade-offs

- **Accessibility**: Screen readers can't "read" a canvas (it's one big image)
- **Load Time**: 2-10MB WASM binary before anything appears
- **Learning Curve**: Rust has a steeper learning curve than JS/React

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (like webpack for Rust)
cargo install trunk
```

### Development

```bash
# Run dev server (hot reload)
trunk serve

# Open http://127.0.0.1:8080
```

### Build for Production

```bash
# Build optimized WASM
trunk build --release

# Output in ./dist/
```

### Build Native Desktop App

```bash
# macOS / Linux / Windows
cargo build --release

# Run
./target/release/flownode_canvas
```

## Project Structure

```
flownode-canvas/
├── src/
│   ├── main.rs      # Entry points (web + native)
│   ├── app.rs       # Main application & UI layout
│   ├── nodes.rs     # Node types and definitions
│   └── graph.rs     # Node graph state & rendering
├── assets/          # Static assets
├── index.html       # The ONE HTML file
├── Cargo.toml       # Rust dependencies
└── Trunk.toml       # Build configuration
```

## Node Types

### Input
- **Image Input** - Load images from file
- **Color** - Solid color picker
- **Number** - Numeric value

### Adjustments
- **Brightness/Contrast**
- **Hue/Saturation**
- **Levels**

### Filters
- **Blur** (Gaussian, Box, Motion)
- **Sharpen**
- **Noise**

### Combine
- **Blend** (Normal, Multiply, Screen, Overlay, etc.)
- **Mask**

### Output
- **Output** - Final render target

## Roadmap

- [ ] Implement actual image processing (currently UI only)
- [ ] wgpu backend for GPU-accelerated processing
- [ ] File import/export
- [ ] Undo/redo
- [ ] Copy/paste nodes
- [ ] Node groups
- [ ] Custom nodes (scripting)
- [ ] AI nodes (Stable Diffusion, etc.)

## Tech Stack Deep Dive

### egui

[egui](https://github.com/emilk/egui) is an immediate-mode GUI library. Unlike React's retained mode (describe what UI should look like, framework figures out updates), immediate mode redraws everything every frame. This sounds expensive but is actually faster for complex, frequently-updating UIs like node editors.

### Trunk

[Trunk](https://trunkrs.dev/) is the Rust equivalent of webpack/vite for WASM apps. It:
- Compiles Rust to WASM
- Processes `index.html` for asset linking
- Runs wasm-opt for size optimization
- Serves with hot reload

### Why Not Leptos/Yew?

These are Rust frameworks that work like React (components, signals, hooks) but run in WASM. The problem: you still can't use React Flow easily. Wrapping React Flow (JS) to talk to Leptos (Rust) gets messy. Going full egui means everything is Rust-native.

## License

MIT
