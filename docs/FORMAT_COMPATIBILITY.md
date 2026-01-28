# FlowNode Format Compatibility

## Goal
The WebAssembly version must be able to load/save workflows in the same format as the existing FlowNode.io (React Flow based), enabling:
1. Load existing workflows from cloud
2. Save workflows that can be opened in FlowNode.io
3. Seamless migration path

## Existing FlowNode.io Format (React Flow)

```json
{
  "nodes": [
    {
      "id": "uuid-string",
      "type": "text|llm|omni|image|video|content|composition|batch|router|convertor|mind-map|image-bucket|groupProxy",
      "position": { "x": 100, "y": 200 },
      "data": {
        "label": "Node Name",
        "isOff": false,
        "loading": false,
        // ... type-specific fields
      }
    }
  ],
  "edges": [
    {
      "id": "edge-uuid",
      "source": "node-uuid",
      "target": "node-uuid",
      "sourceHandle": "output-0",
      "targetHandle": "input-0"
    }
  ],
  "viewport": { "x": 0, "y": 0, "zoom": 1 }
}
```

## WASM Node Type Mapping

| WASM Type | FlowNode.io Equivalent | Notes |
|-----------|----------------------|-------|
| `image_input` | `image` / `content` | Content input node |
| `brightness_contrast` | `image` + adjustment data | Image node with processing |
| `hue_saturation` | `image` + adjustment data | Image node with processing |
| `blur` | `image` + filter data | Image node with processing |
| `output` | `image` | Display output |

## Implementation Strategy

### Phase 1: Read Compatibility
- Parse React Flow JSON format
- Map `type` strings to WASM NodeType enum
- Map `position` to Vec2
- Map `edges` to our Connection struct
- Preserve unknown fields for round-trip compatibility

### Phase 2: Write Compatibility
- Export in React Flow JSON format
- Include `sourceHandle`/`targetHandle` on edges
- Preserve `data` field structure
- Include `viewport` from pan/zoom state

### Phase 3: Live Sync (Future)
- WebSocket/Supabase realtime integration
- Conflict resolution with existing FlowNode clients
- Presence and collaboration support

## Node Data Field Mapping

### Image Node (FlowNode.io)
```json
{
  "type": "image",
  "data": {
    "image": "url-or-base64",
    "history": ["url1", "url2"],
    "isExpanded": false
  }
}
```

### Processing Nodes (WASM)
Store processing parameters in `data.processing`:
```json
{
  "type": "image",
  "data": {
    "image": "...",
    "processing": {
      "type": "brightness_contrast",
      "brightness": 0.1,
      "contrast": 1.2
    }
  }
}
```

## Handle ID Convention

FlowNode.io uses handle IDs like:
- Inputs: `input-0`, `input-1`, etc.
- Outputs: `output-0`, `output-1`, etc.

WASM should use the same convention for compatibility.

## Migration Notes

When loading a FlowNode.io workflow:
1. Parse JSON with serde
2. Check for processing-capable nodes (have adjustment/filter settings)
3. Create appropriate WASM nodes
4. Map edges using handle indices

When saving:
1. Convert WASM nodes to React Flow format
2. Include all original `data` fields
3. Add processing info to `data.processing`
4. Export with proper handle IDs
