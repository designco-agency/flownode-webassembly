# Rust WASM Deployment & Optimization Guide

> Research compiled: January 2026

---

## 1. WASM Binary Size Optimization Techniques

### Cargo.toml Configuration
```toml
[profile.release]
lto = true              # Link-Time Optimization (biggest impact)
opt-level = 's'         # Optimize for size ('z' for aggressive)
codegen-units = 1       # Single codegen unit for better optimization
panic = 'abort'         # Remove panic unwinding code
strip = true            # Strip symbols
```

### wasm-opt Post-Processing
```bash
# Install Binaryen toolkit
# Run after wasm-pack build

wasm-opt -Oz -o output.wasm input.wasm  # Aggressive size optimization
wasm-opt -Os -o output.wasm input.wasm  # Balanced size optimization
```
**Impact:** 15-20% additional size reduction on top of LLVM optimizations

### Code-Level Optimizations

| Technique | Impact | Trade-off |
|-----------|--------|-----------|
| Avoid `format!`, `to_string` | High | Use static strings in release |
| Avoid panics | High | Use `abort()` instead of unwrap |
| Use `get()` over indexing | Medium | Returns `Option` instead of panic |
| Trait objects over generics | Medium | Runtime dispatch vs monomorphization |
| Avoid `std` allocation | High | Use `wee_alloc` (saves ~10KB) |

### Size Profiling with Twiggy
```bash
cargo install twiggy
twiggy top -n 20 pkg/your_module_bg.wasm
twiggy dominators pkg/your_module_bg.wasm
```

### Practical Recommendations
1. **Start with Cargo config** - LTO + opt-level 's' gives best ROI
2. **Always run wasm-opt** - Integrate into build pipeline
3. **Profile before optimizing** - Use twiggy to find bloat sources
4. **Test both 's' and 'z'** - Sometimes 's' produces smaller binaries
5. **Strip debug info** - wasm-pack does this by default

---

## 2. Code Splitting Strategies for Rust WASM

### Module-Level Splitting
```rust
// Split into separate crates
// main_app/Cargo.toml
[dependencies]
core_module = { path = "../core" }
heavy_feature = { path = "../heavy", optional = true }

[features]
default = []
advanced = ["heavy_feature"]
```

### Dynamic Import Pattern (JS-side)
```javascript
// Load core WASM immediately
const core = await import('./pkg/core.js');
await core.default();

// Lazy load heavy modules on demand
async function loadAdvancedFeatures() {
  const advanced = await import('./pkg/advanced.js');
  await advanced.default();
  return advanced;
}
```

### Feature-Gated Compilation
```rust
#[cfg(feature = "webgpu")]
mod webgpu_renderer;

#[cfg(feature = "webgl")]
mod webgl_renderer;

// Build separate .wasm files for each feature set
```

### Practical Recommendations
1. **Identify heavy dependencies** - Separate GPU code, math libs, etc.
2. **Use Cargo features** - Build multiple optimized variants
3. **Separate by usage frequency** - Core vs. rarely-used features
4. **Consider multiple .wasm files** - Trade HTTP requests for smaller initial load

---

## 3. Lazy Loading Modules

### Dynamic WASM Instantiation
```javascript
class WasmModuleLoader {
  constructor() {
    this.modules = new Map();
  }

  async load(moduleName) {
    if (this.modules.has(moduleName)) {
      return this.modules.get(moduleName);
    }

    const module = await import(`./pkg/${moduleName}.js`);
    await module.default(); // Initialize WASM
    this.modules.set(moduleName, module);
    return module;
  }
}

// Usage
const loader = new WasmModuleLoader();

// Load on demand
document.getElementById('advancedBtn').onclick = async () => {
  const advanced = await loader.load('advanced_features');
  advanced.doHeavyComputation();
};
```

### Streaming Compilation for Large Modules
```javascript
// Use instantiateStreaming for fastest load
const response = fetch('./pkg/heavy_module_bg.wasm');
const { instance, module } = await WebAssembly.instantiateStreaming(
  response,
  importObject
);
```

### Prefetch Hints
```html
<!-- Prefetch likely-needed modules during idle time -->
<link rel="prefetch" href="./pkg/gpu_renderer_bg.wasm" as="fetch" crossorigin>
<link rel="modulepreload" href="./pkg/gpu_renderer.js">
```

### Practical Recommendations
1. **Use `instantiateStreaming`** - Compiles as bytes arrive
2. **Prefetch predictable modules** - User clicks settings → prefetch settings module
3. **Idle-time loading** - Use `requestIdleCallback` for speculative loading
4. **Show loading indicators** - WASM modules may take 100ms+ to load

---

## 4. Web Workers for Parallel Processing

### Basic Worker Setup with WASM
```javascript
// main.js
const worker = new Worker(
  new URL('./worker.js', import.meta.url),
  { type: 'module' }
);

worker.postMessage({ type: 'init' });

worker.onmessage = (e) => {
  if (e.data.type === 'result') {
    console.log('Computation result:', e.data.value);
  }
};

// Offload heavy computation
worker.postMessage({ 
  type: 'compute', 
  data: largeDataArray 
});
```

```javascript
// worker.js
import init, { heavy_computation } from './pkg/your_module.js';

let wasmReady = false;

self.onmessage = async (e) => {
  if (e.data.type === 'init') {
    await init();
    wasmReady = true;
    self.postMessage({ type: 'ready' });
  }
  
  if (e.data.type === 'compute' && wasmReady) {
    const result = heavy_computation(e.data.data);
    self.postMessage({ type: 'result', value: result });
  }
};
```

### Transferable Objects for Zero-Copy
```javascript
// Main thread - transfer ownership (zero-copy)
const buffer = new ArrayBuffer(1024 * 1024); // 1MB
worker.postMessage({ data: buffer }, [buffer]);
// buffer is now unusable in main thread

// Worker - return results with transfer
const resultBuffer = new ArrayBuffer(1024);
self.postMessage({ result: resultBuffer }, [resultBuffer]);
```

### Worker Pool Pattern
```javascript
class WasmWorkerPool {
  constructor(size = navigator.hardwareConcurrency) {
    this.workers = [];
    this.queue = [];
    this.available = [];

    for (let i = 0; i < size; i++) {
      const worker = new Worker(new URL('./wasm-worker.js', import.meta.url), { type: 'module' });
      this.workers.push(worker);
      this.available.push(i);
    }
  }

  async execute(task) {
    return new Promise((resolve, reject) => {
      if (this.available.length > 0) {
        const idx = this.available.pop();
        this.runTask(idx, task, resolve, reject);
      } else {
        this.queue.push({ task, resolve, reject });
      }
    });
  }

  runTask(workerIdx, task, resolve, reject) {
    const worker = this.workers[workerIdx];
    const handler = (e) => {
      worker.removeEventListener('message', handler);
      this.available.push(workerIdx);
      this.processQueue();
      resolve(e.data);
    };
    worker.addEventListener('message', handler);
    worker.postMessage(task);
  }

  processQueue() {
    if (this.queue.length > 0 && this.available.length > 0) {
      const { task, resolve, reject } = this.queue.shift();
      const idx = this.available.pop();
      this.runTask(idx, task, resolve, reject);
    }
  }
}
```

### Practical Recommendations
1. **Pool workers** - Reuse workers, don't create/destroy per task
2. **Transfer, don't copy** - Use Transferable objects for large buffers
3. **Batch small tasks** - Worker communication has overhead (~1ms)
4. **Match pool size to cores** - `navigator.hardwareConcurrency`
5. **Initialize WASM once per worker** - Cache the module instance

---

## 5. SharedArrayBuffer and Threading in Rust WASM

### Prerequisites & Security Headers
```
# Required HTTP headers for SharedArrayBuffer
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

### wasm-bindgen-rayon Setup

**Cargo.toml:**
```toml
[dependencies]
wasm-bindgen = "0.2"
rayon = "1.8"
wasm-bindgen-rayon = "1.2"

# For no-bundler usage
# wasm-bindgen-rayon = { version = "1.2", features = ["no-bundler"] }
```

**rust-toolchain.toml:**
```toml
[toolchain]
channel = "nightly-2024-08-02"
components = ["rust-src"]
targets = ["wasm32-unknown-unknown"]
```

**.cargo/config.toml:**
```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+atomics,+bulk-memory"]

[unstable]
build-std = ["panic_abort", "std"]
```

**Rust code:**
```rust
use wasm_bindgen::prelude::*;
use rayon::prelude::*;

// Re-export for JS initialization
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn parallel_sum(numbers: &[i32]) -> i32 {
    numbers.par_iter().sum()
}

#[wasm_bindgen]
pub fn parallel_map(data: &[f64]) -> Vec<f64> {
    data.par_iter()
        .map(|x| x * x + 2.0 * x + 1.0)
        .collect()
}
```

**JavaScript initialization:**
```javascript
import init, { initThreadPool, parallel_sum } from './pkg/your_module.js';

async function main() {
  await init();
  await initThreadPool(navigator.hardwareConcurrency);
  
  const numbers = new Int32Array([1, 2, 3, 4, 5, 6, 7, 8]);
  const result = parallel_sum(numbers);
  console.log('Parallel sum:', result);
}
```

### Feature Detection & Fallback
```javascript
import { threads } from 'wasm-feature-detect';

let wasmModule;

async function initWasm() {
  if (await threads()) {
    // Threaded version
    wasmModule = await import('./pkg-threads/index.js');
    await wasmModule.default();
    await wasmModule.initThreadPool(navigator.hardwareConcurrency);
  } else {
    // Single-threaded fallback
    wasmModule = await import('./pkg-single/index.js');
    await wasmModule.default();
  }
  return wasmModule;
}
```

### Build Commands
```bash
# With threading support
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory' \
  wasm-pack build --target web -- -Z build-std=panic_abort,std

# Without threading (fallback build)
wasm-pack build --target web
```

### Practical Recommendations
1. **Always provide fallback** - Safari/Firefox support varies
2. **Set COOP/COEP headers** - Required for SharedArrayBuffer
3. **Use fixed nightly** - Threading requires unstable features
4. **Build two versions** - Thread + single-thread fallbacks
5. **Test cross-browser** - Thread behavior differs

---

## 6. Memory Management Best Practices

### Rust-side Memory Patterns
```rust
use wasm_bindgen::prelude::*;

// GOOD: Let JavaScript own the memory
#[wasm_bindgen]
pub fn process_data(input: &[u8]) -> Vec<u8> {
    input.iter().map(|x| x * 2).collect()
}

// GOOD: Reuse allocations
#[wasm_bindgen]
pub struct Processor {
    buffer: Vec<u8>,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Self {
        Processor {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn process(&mut self, input: &[u8]) -> *const u8 {
        self.buffer.clear();
        self.buffer.extend(input.iter().map(|x| x * 2));
        self.buffer.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
```

### JavaScript-side Memory Access
```javascript
import init, { Processor, memory } from './pkg/your_module.js';

await init();

const processor = new Processor(1024 * 1024); // 1MB buffer

function processLargeData(inputArray) {
  const inputPtr = processor.process(inputArray);
  const outputLen = processor.len();
  
  // Direct view into WASM memory (zero-copy read)
  const wasmMemory = new Uint8Array(memory.buffer);
  const output = wasmMemory.slice(inputPtr, inputPtr + outputLen);
  
  return output;
}

// Clean up when done
processor.free();
```

### Memory Growth Handling
```javascript
// WASM memory can grow - views become detached
let wasmMemory = new Uint8Array(memory.buffer);

function safeMemoryAccess(ptr, len) {
  // Re-create view if memory grew
  if (wasmMemory.buffer !== memory.buffer) {
    wasmMemory = new Uint8Array(memory.buffer);
  }
  return wasmMemory.subarray(ptr, ptr + len);
}
```

### wee_alloc for Size Optimization
```rust
// Cargo.toml
[dependencies]
wee_alloc = "0.4"

// lib.rs
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```
**Trade-off:** Saves ~10KB but slower allocation

### Practical Recommendations
1. **Reuse buffers** - Avoid repeated allocations
2. **Pre-allocate** - Know your sizes when possible
3. **Use TypedArrays** - Direct memory views, zero-copy
4. **Watch for detached views** - Memory growth invalidates old views
5. **Call `.free()`** - Rust objects need explicit cleanup in JS
6. **Profile memory** - Use browser devtools Memory tab

---

## 7. Progressive Loading UX Patterns

### Loading State Machine
```typescript
type LoadingState = 
  | { phase: 'initial' }
  | { phase: 'core-loading', progress: number }
  | { phase: 'core-ready' }
  | { phase: 'feature-loading', feature: string, progress: number }
  | { phase: 'ready' };

class AppLoader {
  state: LoadingState = { phase: 'initial' };
  
  async load() {
    this.setState({ phase: 'core-loading', progress: 0 });
    
    // Load core WASM with progress
    const coreModule = await this.loadWithProgress(
      './pkg/core_bg.wasm',
      (p) => this.setState({ phase: 'core-loading', progress: p })
    );
    
    this.setState({ phase: 'core-ready' });
    
    // App is usable now - additional features load in background
    return coreModule;
  }
  
  async loadWithProgress(url, onProgress) {
    const response = await fetch(url);
    const contentLength = +response.headers.get('Content-Length');
    const reader = response.body.getReader();
    const chunks = [];
    let received = 0;
    
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
      received += value.length;
      onProgress(received / contentLength);
    }
    
    const blob = new Blob(chunks);
    return WebAssembly.instantiateStreaming(
      new Response(blob),
      importObject
    );
  }
}
```

### Skeleton UI During Load
```css
.wasm-loading {
  background: linear-gradient(
    90deg,
    #f0f0f0 25%,
    #e0e0e0 50%,
    #f0f0f0 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
```

### Progressive Enhancement Pattern
```javascript
class RenderEngine {
  constructor() {
    this.backend = null;
  }

  async init() {
    // Start with basic canvas
    this.backend = new Canvas2DBackend();
    this.render(); // Immediate display
    
    // Upgrade to WebGL if available
    try {
      await this.upgradeToWebGL();
    } catch {
      console.log('WebGL unavailable, using Canvas2D');
    }
    
    // Upgrade to WebGPU if available
    try {
      await this.upgradeToWebGPU();
    } catch {
      console.log('WebGPU unavailable');
    }
  }

  async upgradeToWebGL() {
    const { WebGLBackend } = await import('./pkg/webgl.js');
    this.backend = new WebGLBackend();
    this.render();
  }

  async upgradeToWebGPU() {
    if (!navigator.gpu) throw new Error('No WebGPU');
    const { WebGPUBackend } = await import('./pkg/webgpu.js');
    this.backend = new WebGPUBackend();
    this.render();
  }
}
```

### Practical Recommendations
1. **Show progress** - Use `Content-Length` for determinate progress bars
2. **Skeleton screens** - Better than spinners for perceived speed
3. **Progressive enhancement** - Canvas → WebGL → WebGPU
4. **Load core first** - Make app usable ASAP, enhance later
5. **Preload during idle** - `requestIdleCallback` for likely-needed modules
6. **Optimistic UI** - Show predicted state while loading

---

## 8. Service Worker Caching Strategies

### Basic WASM Caching
```javascript
// sw.js
const CACHE_NAME = 'wasm-app-v1';
const WASM_CACHE = 'wasm-modules-v1';

const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/style.css',
  '/app.js',
];

const WASM_ASSETS = [
  '/pkg/core_bg.wasm',
  '/pkg/core.js',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    Promise.all([
      caches.open(CACHE_NAME).then(cache => cache.addAll(STATIC_ASSETS)),
      caches.open(WASM_CACHE).then(cache => cache.addAll(WASM_ASSETS)),
    ])
  );
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  
  // WASM files: cache-first (immutable content)
  if (url.pathname.endsWith('.wasm')) {
    event.respondWith(
      caches.match(event.request).then(cached => {
        return cached || fetch(event.request).then(response => {
          const clone = response.clone();
          caches.open(WASM_CACHE).then(cache => cache.put(event.request, clone));
          return response;
        });
      })
    );
    return;
  }
  
  // JS files: stale-while-revalidate
  if (url.pathname.endsWith('.js')) {
    event.respondWith(
      caches.match(event.request).then(cached => {
        const fetchPromise = fetch(event.request).then(response => {
          const clone = response.clone();
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
          return response;
        });
        return cached || fetchPromise;
      })
    );
    return;
  }
});
```

### Versioned WASM Caching
```javascript
// Use content hash in filename for cache busting
// core_bg.a1b2c3d4.wasm

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames
          .filter(name => name.startsWith('wasm-') && name !== WASM_CACHE)
          .map(name => caches.delete(name))
      );
    })
  );
});
```

### Background Module Updates
```javascript
// Check for updates without blocking
async function checkForUpdates() {
  const registration = await navigator.serviceWorker.ready;
  await registration.update();
}

// Prompt user when update available
navigator.serviceWorker.addEventListener('controllerchange', () => {
  if (confirm('New version available. Reload?')) {
    window.location.reload();
  }
});
```

### Practical Recommendations
1. **Cache-first for .wasm** - Files are immutable (use hashed filenames)
2. **Stale-while-revalidate for .js** - Balance freshness and speed
3. **Version your caches** - Clean up old versions on activate
4. **Precache critical WASM** - Install event for core modules
5. **Lazy cache optional WASM** - Cache on first fetch for features

---

## 9. CDN Deployment Considerations

### MIME Types & Headers
```nginx
# nginx configuration
location ~ \.wasm$ {
    add_header Content-Type application/wasm;
    add_header Cache-Control "public, max-age=31536000, immutable";
    
    # For threading support
    add_header Cross-Origin-Opener-Policy same-origin;
    add_header Cross-Origin-Embedder-Policy require-corp;
    
    # Compression (optional - WASM compresses well)
    gzip_static on;
    brotli_static on;
}
```

### CDN Configuration Checklist
| Setting | Value | Reason |
|---------|-------|--------|
| Content-Type | `application/wasm` | Required for streaming compilation |
| Cache-Control | `max-age=31536000, immutable` | WASM files are versioned |
| CORS | `Access-Control-Allow-Origin: *` | For cross-origin loading |
| Compression | Brotli > gzip | 20-30% smaller than gzip |
| HTTP/2 or HTTP/3 | Enabled | Parallel loading of modules |

### Multi-Region Deployment
```javascript
// Use multiple CDN origins for resilience
const CDN_ORIGINS = [
  'https://cdn1.example.com',
  'https://cdn2.example.com',
  'https://cdn3.example.com',
];

async function fetchWithFallback(path) {
  for (const origin of CDN_ORIGINS) {
    try {
      const response = await fetch(`${origin}${path}`, { 
        mode: 'cors',
        credentials: 'omit'
      });
      if (response.ok) return response;
    } catch (e) {
      console.warn(`CDN ${origin} failed:`, e);
    }
  }
  throw new Error('All CDNs failed');
}
```

### Precompressed Assets
```bash
# Pre-compress WASM files for CDN
brotli -q 11 pkg/core_bg.wasm -o pkg/core_bg.wasm.br
gzip -9 -k pkg/core_bg.wasm

# Upload both compressed versions
# CDN serves appropriate version based on Accept-Encoding
```

### Practical Recommendations
1. **Use content hashes** - `core_bg.a1b2c3.wasm` for immutable caching
2. **Pre-compress** - Brotli at max compression, serve statically
3. **Set correct MIME** - Required for `instantiateStreaming`
4. **Enable HTTP/2+** - Multiple small modules benefit from multiplexing
5. **Consider edge compute** - Cloudflare Workers can transform responses
6. **Set COOP/COEP at edge** - For SharedArrayBuffer support

---

## 10. Browser Compatibility Matrix

### WebGPU Support (as of January 2026)

| Browser | Desktop | Mobile | Notes |
|---------|---------|--------|-------|
| **Chrome** | ✅ 113+ | ✅ Android 113+ | Full support |
| **Edge** | ✅ 113+ | ✅ | Chromium-based |
| **Safari** | ◐ 26+ | ✅ iOS 26+ | Partial (improving) |
| **Firefox** | ❌ Flag only | ❌ Flag only | `dom.webgpu.enabled` |
| **Opera** | ✅ 99+ | ✅ 80+ | Chromium-based |
| **Samsung** | - | ✅ 24+ | Android |

**WebGPU Coverage:** ~75% of users (Chromium dominance)

### WebGL2 Support

| Browser | Desktop | Mobile | Notes |
|---------|---------|--------|-------|
| **Chrome** | ✅ 56+ | ✅ | Universal |
| **Edge** | ✅ 79+ | ✅ | Universal |
| **Safari** | ✅ 15+ | ✅ iOS 15+ | Previously flag |
| **Firefox** | ✅ 51+ | ✅ | Universal |
| **Opera** | ✅ 43+ | ✅ 80+ | Universal |
| **IE** | ❌ | - | Never supported |

**WebGL2 Coverage:** ~97% of users

### SharedArrayBuffer (WASM Threading)

| Browser | Support | Notes |
|---------|---------|-------|
| **Chrome** | ✅ 68+ | Requires COOP/COEP headers |
| **Edge** | ✅ 79+ | Requires COOP/COEP headers |
| **Safari** | ✅ 15.2+ | Requires COOP/COEP headers |
| **Firefox** | ✅ 79+ | Requires COOP/COEP headers |
| **Opera** | ✅ 64+ | Requires COOP/COEP headers |
| **Android Browser** | ❌ | Not supported |
| **Opera Mini** | ❌ | Not supported |

**Threading Coverage:** ~92% of users (with correct headers)

### WASM Core Features

| Feature | Support | Notes |
|---------|---------|-------|
| **Basic WASM** | 96%+ | All modern browsers |
| **Streaming compilation** | 95%+ | `instantiateStreaming` |
| **SIMD** | 91% | Chrome 91+, Firefox 89+, Safari 16.4+ |
| **Bulk memory** | 94% | Required for threading |
| **Reference types** | 90% | For better JS interop |

### Feature Detection Code
```javascript
async function detectCapabilities() {
  const { simd, threads, bulkMemory } = await import('wasm-feature-detect');
  
  return {
    webgpu: !!navigator.gpu,
    webgl2: (() => {
      const canvas = document.createElement('canvas');
      return !!canvas.getContext('webgl2');
    })(),
    webgl1: (() => {
      const canvas = document.createElement('canvas');
      return !!canvas.getContext('webgl');
    })(),
    threads: await threads(),
    simd: await simd(),
    bulkMemory: await bulkMemory(),
    sharedArrayBuffer: typeof SharedArrayBuffer !== 'undefined',
    serviceWorker: 'serviceWorker' in navigator,
  };
}

// Use capabilities to select optimal code path
const caps = await detectCapabilities();

if (caps.webgpu && caps.threads) {
  // Best path: WebGPU + multithreading
  await loadModule('pkg-webgpu-threaded');
} else if (caps.webgl2 && caps.threads) {
  // Good path: WebGL2 + multithreading
  await loadModule('pkg-webgl2-threaded');
} else if (caps.webgl2) {
  // Fallback: WebGL2 single-threaded
  await loadModule('pkg-webgl2-single');
} else {
  // Minimal: WebGL1 or Canvas
  await loadModule('pkg-fallback');
}
```

### Practical Recommendations
1. **Target WebGL2 as baseline** - 97% coverage
2. **Enhance with WebGPU** - Feature-detect and upgrade
3. **Build threading as optional** - Not all browsers/contexts support it
4. **Use wasm-feature-detect** - Reliable runtime detection
5. **Test Safari specifically** - Often has quirks
6. **Consider iOS Safari carefully** - Significant mobile market

---

## Quick Reference: Optimal Build Configuration

### Cargo.toml
```toml
[package]
name = "your-wasm-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"  # Better panic messages

[profile.release]
lto = true
opt-level = 's'
codegen-units = 1
panic = 'abort'
strip = true

[profile.release.package."*"]
opt-level = 's'
```

### Build Script
```bash
#!/bin/bash
set -e

# Build optimized WASM
wasm-pack build --target web --release

# Post-process with wasm-opt
wasm-opt -Oz -o pkg/app_bg_opt.wasm pkg/app_bg.wasm
mv pkg/app_bg_opt.wasm pkg/app_bg.wasm

# Pre-compress for CDN
brotli -q 11 -f pkg/app_bg.wasm
gzip -9 -f -k pkg/app_bg.wasm

# Report sizes
echo "Original: $(stat -f%z pkg/app_bg.wasm) bytes"
echo "Brotli:   $(stat -f%z pkg/app_bg.wasm.br) bytes"
echo "Gzip:     $(stat -f%z pkg/app_bg.wasm.gz) bytes"
```

---

*This guide covers production-ready patterns for Rust WASM deployment. Adjust recommendations based on your specific use case and target audience.*
