# Egui Node Graph Implementations - Research & Best Practices

## 1. Library Comparison

### egui-snarl (Recommended for Node Editors)
**Repo:** https://github.com/zakarumych/egui-snarl  
**Downloads:** ~31k | **Latest:** 0.9.0

**Strengths:**
- Purpose-built for node-graph editors (visual programming, shader graphs)
- Typed data-only nodes with `Snarl<T>` container
- Rich `SnarlViewer` trait for customization
- Built-in wire drawing with beautiful bezier curves
- Multi-connection support (Shift to bundle, Ctrl to yank)
- Context menus for graph/node operations
- UI scaling support
- Serde serialization out of the box
- Collapsible nodes, custom pin shapes
- Selection rect with configurable behavior

**Best for:** Shader graphs, visual scripting, data flow editors

```rust
// Core structure
pub struct Snarl<T> { /* nodes, positions, wires */ }

// Viewer trait for customization
pub trait SnarlViewer<T> {
    fn title(&mut self, node: &T) -> String;
    fn inputs(&mut self, node: &T) -> usize;
    fn outputs(&mut self, node: &T) -> usize;
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<T>) -> impl SnarlPin;
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<T>) -> impl SnarlPin;
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>);
    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>);
    // ... 25+ more customizable methods
}
```

### egui_graphs (Recommended for Graph Visualization)
**Repo:** https://github.com/blitzar-tech/egui_graphs  
**Downloads:** ~124k | **Latest:** 0.29.0

**Strengths:**
- Built on petgraph for graph algorithms
- Force-directed layouts (Fruchterman-Reingold)
- Hierarchical and random layouts
- Custom display traits for nodes/edges
- Event system for interactions
- Performance-optimized rendering
- Dark/light theme support

**Best for:** Network visualization, dependency graphs, knowledge graphs

```rust
// Works with petgraph
let mut g = petgraph::StableGraph::new();
let graph = egui_graphs::Graph::from(&g);

// Display in UI
ui.add(&mut egui_graphs::GraphView::new(&mut graph)
    .with_interactions(&interaction_settings)
    .with_navigations(&nav_settings)
    .with_styles(&style_settings));
```

### Summary Comparison

| Feature | egui-snarl | egui_graphs |
|---------|------------|-------------|
| Primary Use | Node editors | Graph visualization |
| Data Model | Custom Snarl<T> | petgraph |
| Pin/Slot System | ✅ Rich | ❌ N/A |
| Wire Connections | ✅ Beautiful bezier | ❌ Edges only |
| Layouts | ❌ Manual | ✅ Force-directed, hierarchical |
| Graph Algorithms | ❌ None | ✅ Via petgraph |
| Custom Rendering | ✅ SnarlViewer | ✅ DisplayNode/DisplayEdge |
| Serialization | ✅ Serde | ✅ Serde |
| Selection | ✅ Multi-select | ✅ Multi-select |

---

## 2. Custom Node Rendering in Egui

### egui-snarl Approach (SnarlViewer trait)

```rust
use egui_snarl::{Snarl, SnarlViewer, InPin, OutPin, NodeId};
use egui_snarl::ui::{PinInfo, SnarlStyle, PinShape};

#[derive(Clone)]
enum MyNode {
    Number(f32),
    Add,
    Multiply,
    Output,
}

struct MyViewer;

impl SnarlViewer<MyNode> for MyViewer {
    fn title(&mut self, node: &MyNode) -> String {
        match node {
            MyNode::Number(_) => "Number".into(),
            MyNode::Add => "Add".into(),
            MyNode::Multiply => "Multiply".into(),
            MyNode::Output => "Output".into(),
        }
    }

    fn inputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::Number(_) => 0,
            MyNode::Add | MyNode::Multiply => 2,
            MyNode::Output => 1,
        }
    }

    fn outputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::Output => 0,
            _ => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<MyNode>,
    ) -> PinInfo {
        let node = snarl.get_node(pin.id.node).unwrap();
        
        match node {
            MyNode::Add | MyNode::Multiply => {
                let label = if pin.id.input == 0 { "A" } else { "B" };
                ui.label(label);
                PinInfo::circle().with_fill(egui::Color32::LIGHT_BLUE)
            }
            MyNode::Output => {
                ui.label("Result");
                PinInfo::square().with_fill(egui::Color32::GREEN)
            }
            _ => PinInfo::circle(),
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<MyNode>,
    ) -> PinInfo {
        let node = snarl.get_node(pin.id.node).unwrap();
        
        match node {
            MyNode::Number(value) => {
                // Editable number field
                let mut val = *value;
                if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                    if let Some(n) = snarl.get_node_mut(pin.id.node) {
                        *n = MyNode::Number(val);
                    }
                }
                PinInfo::circle().with_fill(egui::Color32::YELLOW)
            }
            _ => {
                ui.label("Out");
                PinInfo::circle().with_fill(egui::Color32::WHITE)
            }
        }
    }

    // Custom node frame
    fn node_frame(
        &mut self,
        default: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<MyNode>,
    ) -> egui::Frame {
        let node_data = snarl.get_node(node).unwrap();
        let color = match node_data {
            MyNode::Number(_) => egui::Color32::from_rgb(60, 60, 100),
            MyNode::Add => egui::Color32::from_rgb(60, 100, 60),
            MyNode::Multiply => egui::Color32::from_rgb(100, 60, 60),
            MyNode::Output => egui::Color32::from_rgb(100, 100, 60),
        };
        default.fill(color)
    }

    // Custom body content
    fn has_body(&mut self, node: &MyNode) -> bool {
        matches!(node, MyNode::Number(_))
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<MyNode>,
    ) {
        if let Some(MyNode::Number(val)) = snarl.get_node_mut(node) {
            ui.add(egui::Slider::new(val, 0.0..=100.0));
        }
    }
}
```

### egui_graphs Approach (DisplayNode trait)

```rust
use egui_graphs::{DisplayNode, NodeProps, DrawContext};
use egui::{Shape, Pos2, Vec2, Color32};

#[derive(Clone)]
struct CustomNodeShape {
    props: NodeProps<MyNodeData>,
    radius: f32,
}

impl<E, Ty, Ix> DisplayNode<MyNodeData, E, Ty, Ix> for CustomNodeShape
where
    E: Clone,
    Ty: petgraph::EdgeType,
    Ix: petgraph::graph::IndexType,
{
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        let center = self.props.location;
        center + dir.normalized() * self.radius
    }

    fn shapes(&mut self, ctx: &DrawContext<'_>) -> Vec<Shape> {
        let center = ctx.meta.canvas_to_screen_pos(self.props.location);
        let radius = self.radius * ctx.meta.zoom;
        
        let mut shapes = vec![];
        
        // Main circle
        shapes.push(Shape::circle_filled(
            center,
            radius,
            if self.props.selected {
                Color32::LIGHT_BLUE
            } else {
                Color32::DARK_GRAY
            },
        ));
        
        // Label
        if let Some(label) = &self.props.label {
            let text = egui::epaint::text::Fonts::layout_no_wrap(
                ctx.ctx.fonts(),
                label.clone(),
                egui::FontId::default(),
                Color32::WHITE,
            );
            shapes.push(Shape::Text(egui::epaint::TextShape::new(
                center - Vec2::new(text.rect.width() / 2.0, text.rect.height() / 2.0),
                text,
                Color32::WHITE,
            )));
        }
        
        shapes
    }

    fn update(&mut self, state: &NodeProps<MyNodeData>) {
        self.props = state.clone();
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        pos.distance(self.props.location) <= self.radius
    }
}
```

---

## 3. Connection Validation (Type Checking Between Slots)

### Type-Safe Connection Pattern

```rust
use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
    Any, // Accepts any type
}

impl DataType {
    pub fn is_compatible(&self, other: &DataType) -> bool {
        match (self, other) {
            (DataType::Any, _) | (_, DataType::Any) => true,
            (a, b) => a == b,
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self {
            DataType::Float => egui::Color32::from_rgb(150, 200, 150),
            DataType::Vec2 => egui::Color32::from_rgb(150, 150, 200),
            DataType::Vec3 => egui::Color32::from_rgb(200, 150, 150),
            DataType::Vec4 => egui::Color32::from_rgb(200, 200, 150),
            DataType::Color => egui::Color32::from_rgb(200, 150, 200),
            DataType::Texture => egui::Color32::from_rgb(150, 200, 200),
            DataType::Any => egui::Color32::GRAY,
        }
    }
}

#[derive(Clone)]
pub struct TypedPin {
    pub data_type: DataType,
    pub name: String,
}

#[derive(Clone)]
pub struct TypedNode {
    pub name: String,
    pub inputs: Vec<TypedPin>,
    pub outputs: Vec<TypedPin>,
}

// Connection validation in SnarlViewer
impl SnarlViewer<TypedNode> for TypedViewer {
    fn connect(
        &mut self,
        from: &OutPin,
        to: &InPin,
        snarl: &mut Snarl<TypedNode>,
    ) {
        let from_node = snarl.get_node(from.id.node).unwrap();
        let to_node = snarl.get_node(to.id.node).unwrap();
        
        let from_type = &from_node.outputs[from.id.output].data_type;
        let to_type = &to_node.inputs[to.id.input].data_type;
        
        if from_type.is_compatible(to_type) {
            // Valid connection - proceed
            snarl.connect(from.id, to.id);
        } else {
            // Invalid - could show error tooltip or visual feedback
            log::warn!(
                "Cannot connect {:?} to {:?}",
                from_type, to_type
            );
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<TypedNode>,
    ) -> PinInfo {
        let node = snarl.get_node(pin.id.node).unwrap();
        let pin_data = &node.inputs[pin.id.input];
        
        ui.label(&pin_data.name);
        
        PinInfo::circle()
            .with_fill(pin_data.data_type.color())
            .with_stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
    }
}
```

### Advanced Type System with Generics

```rust
pub trait PinType: 'static {
    fn type_id() -> TypeId { TypeId::of::<Self>() }
    fn type_name() -> &'static str;
    fn color() -> Color32;
}

impl PinType for f32 {
    fn type_name() -> &'static str { "Float" }
    fn color() -> Color32 { Color32::from_rgb(150, 200, 150) }
}

impl PinType for [f32; 3] {
    fn type_name() -> &'static str { "Vec3" }
    fn color() -> Color32 { Color32::from_rgb(200, 150, 150) }
}

pub struct TypedConnection {
    type_id: TypeId,
    type_name: &'static str,
}

impl TypedConnection {
    pub fn new<T: PinType>() -> Self {
        Self {
            type_id: T::type_id(),
            type_name: T::type_name(),
        }
    }
    
    pub fn can_connect(&self, other: &TypedConnection) -> bool {
        self.type_id == other.type_id
    }
}
```

---

## 4. Graph Evaluation / Execution Order (Topological Sorting)

### Using petgraph for Topological Sort

```rust
use petgraph::algo::toposort;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

pub struct GraphEvaluator<T> {
    graph: StableGraph<T, ()>,
    cache: HashMap<NodeIndex, Value>,
}

impl<T: NodeBehavior> GraphEvaluator<T> {
    pub fn evaluate(&mut self) -> Result<HashMap<NodeIndex, Value>, EvalError> {
        // Get topological order
        let order = toposort(&self.graph, None)
            .map_err(|cycle| EvalError::CycleDetected(cycle.node_id()))?;
        
        self.cache.clear();
        
        // Evaluate in order
        for node_idx in order {
            let node = &self.graph[node_idx];
            
            // Gather inputs from cached outputs
            let inputs: Vec<Value> = self.graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .map(|edge| {
                    self.cache.get(&edge.source())
                        .cloned()
                        .unwrap_or(Value::None)
                })
                .collect();
            
            // Evaluate node
            let output = node.evaluate(&inputs)?;
            self.cache.insert(node_idx, output);
        }
        
        Ok(self.cache.clone())
    }
}

pub trait NodeBehavior {
    fn evaluate(&self, inputs: &[Value]) -> Result<Value, EvalError>;
}

#[derive(Clone)]
pub enum Value {
    None,
    Float(f32),
    Vec3([f32; 3]),
    // ...
}
```

### Manual Topological Sort for egui-snarl

```rust
use egui_snarl::{Snarl, NodeId, InPinId, OutPinId};
use std::collections::{HashMap, HashSet, VecDeque};

pub fn topological_sort<T>(snarl: &Snarl<T>) -> Result<Vec<NodeId>, NodeId> {
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    
    // Initialize
    for (id, _) in snarl.nodes_ids_data() {
        in_degree.insert(id, 0);
        adjacency.insert(id, Vec::new());
    }
    
    // Build graph from wires
    for (from, to) in snarl.wires() {
        let from_node = from.node;
        let to_node = to.node;
        
        adjacency.get_mut(&from_node).unwrap().push(to_node);
        *in_degree.get_mut(&to_node).unwrap() += 1;
    }
    
    // Kahn's algorithm
    let mut queue: VecDeque<NodeId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    let mut result = Vec::new();
    
    while let Some(node) = queue.pop_front() {
        result.push(node);
        
        for &neighbor in adjacency.get(&node).unwrap() {
            let deg = in_degree.get_mut(&neighbor).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(neighbor);
            }
        }
    }
    
    // Check for cycles
    if result.len() != in_degree.len() {
        // Find a node still with edges (part of cycle)
        let cycle_node = in_degree
            .iter()
            .find(|(_, &deg)| deg > 0)
            .map(|(&id, _)| id)
            .unwrap();
        return Err(cycle_node);
    }
    
    Ok(result)
}
```

### Lazy Evaluation with Caching

```rust
pub struct LazyEvaluator<T> {
    dirty: HashSet<NodeId>,
    cache: HashMap<NodeId, Value>,
}

impl<T: NodeBehavior> LazyEvaluator<T> {
    pub fn mark_dirty(&mut self, node: NodeId, snarl: &Snarl<T>) {
        self.dirty.insert(node);
        
        // Propagate to dependents
        for (from, to) in snarl.wires() {
            if from.node == node {
                self.mark_dirty(to.node, snarl);
            }
        }
    }
    
    pub fn evaluate_node(
        &mut self,
        node: NodeId,
        snarl: &Snarl<T>,
    ) -> Result<Value, EvalError> {
        if !self.dirty.contains(&node) {
            if let Some(cached) = self.cache.get(&node) {
                return Ok(cached.clone());
            }
        }
        
        // Evaluate dependencies first
        let inputs = self.gather_inputs(node, snarl)?;
        
        let node_data = snarl.get_node(node).unwrap();
        let result = node_data.evaluate(&inputs)?;
        
        self.cache.insert(node, result.clone());
        self.dirty.remove(&node);
        
        Ok(result)
    }
}
```

---

## 5. Undo/Redo Patterns for Graph Editors

### Using the `undo` Crate

```rust
use undo::{Edit, Record};

// Define edit commands
pub enum GraphEdit {
    AddNode {
        id: NodeId,
        node: MyNode,
        pos: egui::Pos2,
    },
    RemoveNode {
        id: NodeId,
        node: MyNode,
        pos: egui::Pos2,
        // Store connections for restoration
        connections: Vec<(OutPinId, InPinId)>,
    },
    Connect {
        from: OutPinId,
        to: InPinId,
    },
    Disconnect {
        from: OutPinId,
        to: InPinId,
    },
    MoveNode {
        id: NodeId,
        old_pos: egui::Pos2,
        new_pos: egui::Pos2,
    },
    ModifyNode {
        id: NodeId,
        old_value: MyNode,
        new_value: MyNode,
    },
}

impl Edit for GraphEdit {
    type Target = Snarl<MyNode>;
    type Output = ();

    fn edit(&mut self, snarl: &mut Self::Target) {
        match self {
            GraphEdit::AddNode { id, node, pos } => {
                *id = snarl.insert_node(*pos, node.clone());
            }
            GraphEdit::RemoveNode { id, .. } => {
                snarl.remove_node(*id);
            }
            GraphEdit::Connect { from, to } => {
                snarl.connect(*from, *to);
            }
            GraphEdit::Disconnect { from, to } => {
                snarl.disconnect(*from, *to);
            }
            GraphEdit::MoveNode { id, new_pos, .. } => {
                if let Some(node) = snarl.get_node_info_mut(*id) {
                    node.set_pos(*new_pos);
                }
            }
            GraphEdit::ModifyNode { id, new_value, .. } => {
                if let Some(node) = snarl.get_node_mut(*id) {
                    *node = new_value.clone();
                }
            }
        }
    }

    fn undo(&mut self, snarl: &mut Self::Target) {
        match self {
            GraphEdit::AddNode { id, .. } => {
                snarl.remove_node(*id);
            }
            GraphEdit::RemoveNode { id, node, pos, connections } => {
                *id = snarl.insert_node(*pos, node.clone());
                for (from, to) in connections {
                    snarl.connect(*from, *to);
                }
            }
            GraphEdit::Connect { from, to } => {
                snarl.disconnect(*from, *to);
            }
            GraphEdit::Disconnect { from, to } => {
                snarl.connect(*from, *to);
            }
            GraphEdit::MoveNode { id, old_pos, .. } => {
                if let Some(node) = snarl.get_node_info_mut(*id) {
                    node.set_pos(*old_pos);
                }
            }
            GraphEdit::ModifyNode { id, old_value, .. } => {
                if let Some(node) = snarl.get_node_mut(*id) {
                    *node = old_value.clone();
                }
            }
        }
    }
}

// Graph editor with undo support
pub struct GraphEditor {
    snarl: Snarl<MyNode>,
    history: Record<GraphEdit>,
}

impl GraphEditor {
    pub fn add_node(&mut self, node: MyNode, pos: egui::Pos2) {
        let mut edit = GraphEdit::AddNode {
            id: NodeId::default(), // Will be set by edit
            node,
            pos,
        };
        self.history.edit(&mut self.snarl, edit);
    }

    pub fn undo(&mut self) {
        self.history.undo(&mut self.snarl);
    }

    pub fn redo(&mut self) {
        self.history.redo(&mut self.snarl);
    }
    
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }
    
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }
}
```

### Snapshot-Based Undo (Simpler but More Memory)

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GraphSnapshot<T: Clone> {
    nodes: Vec<(NodeId, T, egui::Pos2)>,
    wires: Vec<(OutPinId, InPinId)>,
}

pub struct SnapshotHistory<T: Clone> {
    snapshots: Vec<GraphSnapshot<T>>,
    current: usize,
    max_history: usize,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> SnapshotHistory<T> {
    pub fn new(max_history: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            current: 0,
            max_history,
        }
    }

    pub fn save(&mut self, snarl: &Snarl<T>) {
        // Remove any redo history
        self.snapshots.truncate(self.current);
        
        // Create snapshot
        let snapshot = GraphSnapshot {
            nodes: snarl.nodes_pos_ids()
                .map(|(node, pos, id)| (id, node.clone(), pos))
                .collect(),
            wires: snarl.wires().collect(),
        };
        
        self.snapshots.push(snapshot);
        self.current = self.snapshots.len();
        
        // Limit history size
        if self.snapshots.len() > self.max_history {
            self.snapshots.remove(0);
            self.current -= 1;
        }
    }

    pub fn undo(&mut self, snarl: &mut Snarl<T>) -> bool {
        if self.current > 1 {
            self.current -= 1;
            self.restore(snarl);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, snarl: &mut Snarl<T>) -> bool {
        if self.current < self.snapshots.len() {
            self.current += 1;
            self.restore(snarl);
            true
        } else {
            false
        }
    }

    fn restore(&self, snarl: &mut Snarl<T>) {
        let snapshot = &self.snapshots[self.current - 1];
        
        // Clear and rebuild
        *snarl = Snarl::new();
        
        for (id, node, pos) in &snapshot.nodes {
            snarl.insert_node(*pos, node.clone());
        }
        
        for (from, to) in &snapshot.wires {
            snarl.connect(*from, *to);
        }
    }
}
```

---

## 6. Serialization/Deserialization (Serde Patterns)

### Basic Serde Setup for egui-snarl

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum MyNode {
    Number { value: f32 },
    Add,
    Multiply,
    Output { name: String },
}

// Snarl<T> implements Serialize/Deserialize when T does
// Enable with feature flag: egui-snarl = { features = ["serde"] }

pub struct GraphEditor {
    #[serde(skip)] // UI state not serialized
    viewer: MyViewer,
    
    snarl: Snarl<MyNode>,
}

impl GraphEditor {
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.snarl)?;
        std::fs::write(path, json)
    }

    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.snarl = serde_json::from_str(&json)?;
        Ok(())
    }
}
```

### Custom Serialization with Metadata

```rust
#[derive(Serialize, Deserialize)]
pub struct GraphDocument {
    pub version: u32,
    pub name: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub graph: Snarl<MyNode>,
    pub metadata: GraphMetadata,
}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub viewport_offset: [f32; 2],
    pub viewport_zoom: f32,
    pub selected_nodes: Vec<NodeId>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl GraphDocument {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            version: 1,
            name,
            created: now,
            modified: now,
            graph: Snarl::new(),
            metadata: GraphMetadata::default(),
        }
    }

    pub fn save(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.modified = chrono::Utc::now();
        
        // Binary format for smaller files
        let bytes = bincode::serialize(&self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let doc: Self = bincode::deserialize(&bytes)?;
        Ok(doc)
    }
}
```

### Handling Node Type Evolution (Migrations)

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MyNode {
    #[serde(rename = "number")]
    Number { value: f32 },
    
    #[serde(rename = "add")]
    Add,
    
    // New node type - old files won't have this
    #[serde(rename = "subtract")]
    Subtract,
    
    // Deprecated but still loadable
    #[serde(rename = "legacy_mult")]
    #[serde(alias = "multiply")] // Handle old name
    Multiply,
}

// Version migration
pub fn migrate_graph(json: &str) -> Result<Snarl<MyNode>, serde_json::Error> {
    // Try parsing as current version
    if let Ok(graph) = serde_json::from_str::<Snarl<MyNode>>(json) {
        return Ok(graph);
    }
    
    // Try parsing as v1 and migrate
    #[derive(Deserialize)]
    struct V1Node {
        kind: String,
        value: Option<f32>,
    }
    
    // ... migration logic
    todo!()
}
```

---

## 7. Minimap / Overview Implementations

### Basic Minimap for Node Graphs

```rust
pub struct MinimapWidget<'a, T> {
    snarl: &'a Snarl<T>,
    viewport: egui::Rect,
    size: egui::Vec2,
}

impl<'a, T> MinimapWidget<'a, T> {
    pub fn new(snarl: &'a Snarl<T>, viewport: egui::Rect) -> Self {
        Self {
            snarl,
            viewport,
            size: egui::vec2(150.0, 100.0),
        }
    }
    
    pub fn show(self, ui: &mut egui::Ui) -> Option<egui::Pos2> {
        let (response, painter) = ui.allocate_painter(self.size, egui::Sense::click_and_drag());
        let rect = response.rect;
        
        // Calculate bounds of all nodes
        let mut bounds = egui::Rect::NOTHING;
        for (_, pos, _) in self.snarl.nodes_pos_ids() {
            bounds = bounds.union(egui::Rect::from_center_size(pos, egui::vec2(50.0, 30.0)));
        }
        
        // Add padding
        bounds = bounds.expand(50.0);
        
        // Draw background
        painter.rect_filled(rect, 4.0, egui::Color32::from_gray(30));
        
        // Transform functions
        let world_to_minimap = |pos: egui::Pos2| -> egui::Pos2 {
            let normalized = egui::vec2(
                (pos.x - bounds.min.x) / bounds.width(),
                (pos.y - bounds.min.y) / bounds.height(),
            );
            rect.min + egui::vec2(
                normalized.x * rect.width(),
                normalized.y * rect.height(),
            )
        };
        
        // Draw nodes as small dots
        for (_, pos, _) in self.snarl.nodes_pos_ids() {
            let minimap_pos = world_to_minimap(pos);
            painter.circle_filled(minimap_pos, 3.0, egui::Color32::LIGHT_BLUE);
        }
        
        // Draw viewport rectangle
        let viewport_min = world_to_minimap(self.viewport.min);
        let viewport_max = world_to_minimap(self.viewport.max);
        let viewport_rect = egui::Rect::from_min_max(viewport_min, viewport_max);
        
        painter.rect_stroke(
            viewport_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
        );
        
        // Handle click to navigate
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                // Convert minimap position to world position
                let normalized = egui::vec2(
                    (pos.x - rect.min.x) / rect.width(),
                    (pos.y - rect.min.y) / rect.height(),
                );
                let world_pos = egui::pos2(
                    bounds.min.x + normalized.x * bounds.width(),
                    bounds.min.y + normalized.y * bounds.height(),
                );
                return Some(world_pos);
            }
        }
        
        None
    }
}

// Usage
fn show_graph_with_minimap(ui: &mut egui::Ui, snarl: &mut Snarl<MyNode>) {
    // Main graph area
    let main_rect = ui.available_rect_before_wrap();
    
    // ... render main graph, get current viewport
    let viewport = egui::Rect::from_center_size(
        egui::pos2(0.0, 0.0), // current pan offset
        main_rect.size() / 1.0, // zoom factor
    );
    
    // Minimap overlay in corner
    egui::Area::new(egui::Id::new("minimap"))
        .fixed_pos(main_rect.right_bottom() - egui::vec2(160.0, 110.0))
        .show(ui.ctx(), |ui| {
            if let Some(target) = MinimapWidget::new(snarl, viewport).show(ui) {
                // Navigate to target position
            }
        });
}
```

---

## 8. Node Grouping / Subgraphs

### Subgraph as a Node Type

```rust
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub enum MyNode {
    // Regular nodes
    Number { value: f32 },
    Add,
    
    // Subgraph node - contains another graph
    Subgraph {
        id: Uuid,
        name: String,
        // Exposed inputs/outputs
        exposed_inputs: Vec<(String, DataType)>,
        exposed_outputs: Vec<(String, DataType)>,
    },
    
    // Subgraph interface nodes (inside subgraph)
    SubgraphInput { index: usize },
    SubgraphOutput { index: usize },
}

pub struct GraphSystem {
    pub main_graph: Snarl<MyNode>,
    pub subgraphs: HashMap<Uuid, Snarl<MyNode>>,
}

impl GraphSystem {
    pub fn create_subgraph(&mut self, name: String) -> Uuid {
        let id = Uuid::new_v4();
        self.subgraphs.insert(id, Snarl::new());
        id
    }
    
    pub fn create_subgraph_from_selection(
        &mut self,
        graph: &mut Snarl<MyNode>,
        selected: &[NodeId],
        name: String,
    ) -> NodeId {
        let subgraph_id = Uuid::new_v4();
        let mut subgraph = Snarl::new();
        
        // Analyze connections to determine exposed ports
        let mut exposed_inputs = Vec::new();
        let mut exposed_outputs = Vec::new();
        
        // ... analyze selected nodes and their connections
        // ... move nodes to subgraph
        // ... create interface nodes
        
        self.subgraphs.insert(subgraph_id, subgraph);
        
        // Create subgraph node in main graph
        graph.insert_node(
            egui::pos2(0.0, 0.0),
            MyNode::Subgraph {
                id: subgraph_id,
                name,
                exposed_inputs,
                exposed_outputs,
            },
        )
    }
}
```

### Visual Grouping (Frames/Comments)

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    pub id: Uuid,
    pub name: String,
    pub color: [f32; 4],
    pub nodes: HashSet<NodeId>,
    // Cached bounding rect
    #[serde(skip)]
    pub bounds: Option<egui::Rect>,
}

pub struct GraphWithGroups<T> {
    pub snarl: Snarl<T>,
    pub groups: Vec<NodeGroup>,
}

impl<T> GraphWithGroups<T> {
    pub fn update_group_bounds(&mut self) {
        for group in &mut self.groups {
            if group.nodes.is_empty() {
                group.bounds = None;
                continue;
            }
            
            let mut bounds = egui::Rect::NOTHING;
            for (_, pos, id) in self.snarl.nodes_pos_ids() {
                if group.nodes.contains(&id) {
                    // Assume node size
                    let node_rect = egui::Rect::from_center_size(pos, egui::vec2(100.0, 80.0));
                    bounds = bounds.union(node_rect);
                }
            }
            
            group.bounds = Some(bounds.expand(20.0)); // Padding
        }
    }
    
    pub fn draw_group_backgrounds(&self, painter: &egui::Painter, transform: impl Fn(egui::Rect) -> egui::Rect) {
        for group in &self.groups {
            if let Some(bounds) = group.bounds {
                let screen_rect = transform(bounds);
                let color = egui::Color32::from_rgba_unmultiplied(
                    (group.color[0] * 255.0) as u8,
                    (group.color[1] * 255.0) as u8,
                    (group.color[2] * 255.0) as u8,
                    (group.color[3] * 255.0 * 0.3) as u8, // Semi-transparent
                );
                
                painter.rect_filled(screen_rect, 8.0, color);
                
                // Draw label
                painter.text(
                    screen_rect.left_top() + egui::vec2(8.0, 4.0),
                    egui::Align2::LEFT_TOP,
                    &group.name,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}
```

---

## 9. Copy/Paste Between Nodes

### Clipboard System

```rust
use arboard::Clipboard;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardData<T> {
    pub nodes: Vec<(T, egui::Pos2)>,
    pub internal_wires: Vec<(usize, usize, usize, usize)>, // (from_node_idx, from_output, to_node_idx, to_input)
}

pub struct GraphClipboard<T: Clone + Serialize + for<'de> Deserialize<'de>> {
    system_clipboard: Option<Clipboard>,
    internal_buffer: Option<ClipboardData<T>>,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> GraphClipboard<T> {
    pub fn new() -> Self {
        Self {
            system_clipboard: Clipboard::new().ok(),
            internal_buffer: None,
        }
    }

    pub fn copy(&mut self, snarl: &Snarl<T>, selected: &[NodeId]) {
        if selected.is_empty() {
            return;
        }

        // Map old IDs to indices
        let id_to_idx: HashMap<NodeId, usize> = selected.iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        // Collect nodes with positions
        let nodes: Vec<(T, egui::Pos2)> = selected.iter()
            .filter_map(|&id| {
                let node = snarl.get_node(id)?.clone();
                let pos = snarl.get_node_info(id)?.pos();
                Some((node, pos))
            })
            .collect();

        // Find center for relative positioning
        let center = nodes.iter()
            .map(|(_, pos)| pos.to_vec2())
            .reduce(|a, b| a + b)
            .map(|sum| sum / nodes.len() as f32)
            .unwrap_or_default();

        // Adjust positions relative to center
        let nodes: Vec<(T, egui::Pos2)> = nodes.into_iter()
            .map(|(node, pos)| (node, pos - center))
            .collect();

        // Collect internal wires (connections between selected nodes)
        let internal_wires: Vec<_> = snarl.wires()
            .filter_map(|(from, to)| {
                let from_idx = id_to_idx.get(&from.node)?;
                let to_idx = id_to_idx.get(&to.node)?;
                Some((*from_idx, from.output, *to_idx, to.input))
            })
            .collect();

        let data = ClipboardData { nodes, internal_wires };
        self.internal_buffer = Some(data.clone());

        // Also copy to system clipboard as JSON
        if let Some(clipboard) = &mut self.system_clipboard {
            if let Ok(json) = serde_json::to_string(&data) {
                let _ = clipboard.set_text(format!("NODEGRAPH:{}", json));
            }
        }
    }

    pub fn paste(&mut self, snarl: &mut Snarl<T>, paste_pos: egui::Pos2) -> Vec<NodeId> {
        // Try system clipboard first
        let data = self.try_from_system_clipboard()
            .or_else(|| self.internal_buffer.clone());

        let Some(data) = data else {
            return Vec::new();
        };

        // Create nodes
        let new_ids: Vec<NodeId> = data.nodes.iter()
            .map(|(node, rel_pos)| {
                let pos = paste_pos + rel_pos.to_vec2();
                snarl.insert_node(pos, node.clone())
            })
            .collect();

        // Recreate internal wires
        for (from_idx, from_output, to_idx, to_input) in data.internal_wires {
            let from_id = OutPinId {
                node: new_ids[from_idx],
                output: from_output,
            };
            let to_id = InPinId {
                node: new_ids[to_idx],
                input: to_input,
            };
            snarl.connect(from_id, to_id);
        }

        new_ids
    }

    fn try_from_system_clipboard(&mut self) -> Option<ClipboardData<T>> {
        let clipboard = self.system_clipboard.as_mut()?;
        let text = clipboard.get_text().ok()?;
        let json = text.strip_prefix("NODEGRAPH:")?;
        serde_json::from_str(json).ok()
    }

    pub fn cut(&mut self, snarl: &mut Snarl<T>, selected: &[NodeId]) {
        self.copy(snarl, selected);
        for &id in selected {
            snarl.remove_node(id);
        }
    }

    pub fn duplicate(&mut self, snarl: &mut Snarl<T>, selected: &[NodeId]) -> Vec<NodeId> {
        self.copy(snarl, selected);
        
        // Calculate center of selection for offset
        let center: egui::Pos2 = selected.iter()
            .filter_map(|&id| snarl.get_node_info(id).map(|n| n.pos()))
            .fold(egui::Pos2::ZERO, |acc, pos| acc + pos.to_vec2())
            / selected.len() as f32;
        
        // Paste with small offset
        self.paste(snarl, center + egui::vec2(50.0, 50.0))
    }
}
```

---

## 10. Performance Optimization for Large Graphs

### Viewport Culling

```rust
pub struct CulledGraphView<'a, T> {
    snarl: &'a Snarl<T>,
    viewport: egui::Rect,
    node_size_estimate: egui::Vec2,
}

impl<'a, T> CulledGraphView<'a, T> {
    pub fn visible_nodes(&self) -> impl Iterator<Item = (NodeId, &T, egui::Pos2)> {
        // Expand viewport slightly to prevent pop-in
        let expanded = self.viewport.expand2(self.node_size_estimate);
        
        self.snarl.nodes_pos_ids()
            .filter(move |(_, pos, _)| {
                let node_rect = egui::Rect::from_center_size(*pos, self.node_size_estimate);
                expanded.intersects(node_rect)
            })
            .map(|(node, pos, id)| (id, node, pos))
    }
    
    pub fn visible_wires(&self, visible_nodes: &HashSet<NodeId>) -> impl Iterator<Item = (OutPinId, InPinId)> + '_ {
        self.snarl.wires()
            .filter(move |(from, to)| {
                visible_nodes.contains(&from.node) || visible_nodes.contains(&to.node)
            })
    }
}
```

### Level of Detail (LOD)

```rust
pub enum NodeLOD {
    Full,      // All details, interactive
    Reduced,   // No text, simplified shapes
    Dot,       // Just a colored dot
}

impl NodeLOD {
    pub fn from_zoom(zoom: f32) -> Self {
        if zoom > 0.5 {
            NodeLOD::Full
        } else if zoom > 0.2 {
            NodeLOD::Reduced
        } else {
            NodeLOD::Dot
        }
    }
}

pub fn render_node_with_lod(
    painter: &egui::Painter,
    node: &MyNode,
    pos: egui::Pos2,
    lod: NodeLOD,
    zoom: f32,
) {
    match lod {
        NodeLOD::Full => {
            // Full rendering with egui widgets
            // ... normal node rendering
        }
        NodeLOD::Reduced => {
            // Simplified rendering - just shapes, no text
            let size = egui::vec2(80.0, 40.0) * zoom;
            let rect = egui::Rect::from_center_size(pos, size);
            
            painter.rect_filled(rect, 4.0, node.color());
            
            // Simple pin indicators
            for i in 0..node.input_count() {
                let y = rect.top() + (i as f32 + 0.5) * rect.height() / node.input_count() as f32;
                painter.circle_filled(egui::pos2(rect.left(), y), 3.0, egui::Color32::WHITE);
            }
        }
        NodeLOD::Dot => {
            // Minimal - just a colored dot
            painter.circle_filled(pos, 4.0 * zoom.max(0.5), node.color());
        }
    }
}
```

### Spatial Indexing with R-Tree

```rust
use rstar::{RTree, AABB, RTreeObject, PointDistance};

#[derive(Clone)]
struct SpatialNode {
    id: NodeId,
    bounds: AABB<[f32; 2]>,
}

impl RTreeObject for SpatialNode {
    type Envelope = AABB<[f32; 2]>;
    
    fn envelope(&self) -> Self::Envelope {
        self.bounds.clone()
    }
}

pub struct SpatialIndex {
    tree: RTree<SpatialNode>,
}

impl SpatialIndex {
    pub fn rebuild<T>(&mut self, snarl: &Snarl<T>, node_sizes: &HashMap<NodeId, egui::Vec2>) {
        let nodes: Vec<SpatialNode> = snarl.nodes_pos_ids()
            .map(|(_, pos, id)| {
                let size = node_sizes.get(&id).copied().unwrap_or(egui::vec2(100.0, 60.0));
                let half = size / 2.0;
                SpatialNode {
                    id,
                    bounds: AABB::from_corners(
                        [pos.x - half.x, pos.y - half.y],
                        [pos.x + half.x, pos.y + half.y],
                    ),
                }
            })
            .collect();
        
        self.tree = RTree::bulk_load(nodes);
    }
    
    pub fn query_rect(&self, rect: egui::Rect) -> Vec<NodeId> {
        let aabb = AABB::from_corners(
            [rect.min.x, rect.min.y],
            [rect.max.x, rect.max.y],
        );
        
        self.tree.locate_in_envelope(&aabb)
            .map(|node| node.id)
            .collect()
    }
    
    pub fn query_point(&self, pos: egui::Pos2) -> Option<NodeId> {
        self.tree.locate_all_at_point(&[pos.x, pos.y])
            .next()
            .map(|node| node.id)
    }
}
```

### Batch Rendering for Wires

```rust
pub struct WireBatcher {
    vertices: Vec<egui::epaint::Vertex>,
    indices: Vec<u32>,
}

impl WireBatcher {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(10000),
            indices: Vec::with_capacity(15000),
        }
    }
    
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
    
    pub fn add_bezier_wire(
        &mut self,
        from: egui::Pos2,
        to: egui::Pos2,
        color: egui::Color32,
        width: f32,
    ) {
        // Calculate bezier control points
        let dx = (to.x - from.x).abs() * 0.5;
        let cp1 = egui::pos2(from.x + dx, from.y);
        let cp2 = egui::pos2(to.x - dx, to.y);
        
        // Tessellate bezier to line segments
        let segments = 16;
        let base_idx = self.vertices.len() as u32;
        
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let pos = cubic_bezier(from, cp1, cp2, to, t);
            
            // Calculate tangent for perpendicular offset
            let tangent = if i < segments {
                let next_t = (i + 1) as f32 / segments as f32;
                let next_pos = cubic_bezier(from, cp1, cp2, to, next_t);
                (next_pos - pos).normalized()
            } else {
                let prev_t = (i - 1) as f32 / segments as f32;
                let prev_pos = cubic_bezier(from, cp1, cp2, to, prev_t);
                (pos - prev_pos).normalized()
            };
            
            let normal = egui::vec2(-tangent.y, tangent.x) * width * 0.5;
            
            self.vertices.push(egui::epaint::Vertex {
                pos: pos + normal,
                uv: egui::epaint::WHITE_UV,
                color,
            });
            self.vertices.push(egui::epaint::Vertex {
                pos: pos - normal,
                uv: egui::epaint::WHITE_UV,
                color,
            });
            
            if i < segments {
                let idx = base_idx + i as u32 * 2;
                self.indices.extend_from_slice(&[
                    idx, idx + 1, idx + 2,
                    idx + 1, idx + 3, idx + 2,
                ]);
            }
        }
    }
    
    pub fn to_mesh(&self) -> egui::epaint::Mesh {
        egui::epaint::Mesh {
            indices: self.indices.clone(),
            vertices: self.vertices.clone(),
            texture_id: egui::TextureId::Managed(0),
        }
    }
}

fn cubic_bezier(p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2, t: f32) -> egui::Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    egui::pos2(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}
```

### Incremental Updates

```rust
pub struct IncrementalRenderer {
    node_cache: HashMap<NodeId, egui::epaint::Shape>,
    wire_mesh_cache: Option<egui::epaint::Mesh>,
    dirty_nodes: HashSet<NodeId>,
    wires_dirty: bool,
}

impl IncrementalRenderer {
    pub fn mark_node_dirty(&mut self, id: NodeId) {
        self.dirty_nodes.insert(id);
    }
    
    pub fn mark_wires_dirty(&mut self) {
        self.wires_dirty = true;
    }
    
    pub fn render<T: NodeRenderable>(
        &mut self,
        snarl: &Snarl<T>,
        painter: &egui::Painter,
        viewport: egui::Rect,
    ) {
        // Update only dirty nodes
        for id in self.dirty_nodes.drain() {
            if let Some(node) = snarl.get_node(id) {
                if let Some(info) = snarl.get_node_info(id) {
                    let shape = node.render_to_shape(info.pos());
                    self.node_cache.insert(id, shape);
                }
            }
        }
        
        // Update wire mesh if needed
        if self.wires_dirty {
            let mut batcher = WireBatcher::new();
            for (from, to) in snarl.wires() {
                // ... add wires to batcher
            }
            self.wire_mesh_cache = Some(batcher.to_mesh());
            self.wires_dirty = false;
        }
        
        // Draw cached content
        if let Some(mesh) = &self.wire_mesh_cache {
            painter.add(egui::Shape::mesh(mesh.clone()));
        }
        
        for (id, shape) in &self.node_cache {
            // Only draw if in viewport
            if let Some(info) = snarl.get_node_info(*id) {
                let node_rect = egui::Rect::from_center_size(info.pos(), egui::vec2(100.0, 60.0));
                if viewport.intersects(node_rect) {
                    painter.add(shape.clone());
                }
            }
        }
    }
}
```

---

## Summary & Recommendations

### For Visual Programming / Shader Graphs
**Use egui-snarl** - It's purpose-built with:
- Rich pin/slot system with type support
- Beautiful wire rendering
- Full viewer trait for customization
- Built-in serialization

### For Network/Dependency Visualization  
**Use egui_graphs** - It provides:
- petgraph integration for algorithms
- Force-directed and hierarchical layouts
- Efficient rendering for large graphs

### Architecture Best Practices
1. **Separate data from UI** - Keep node logic in a trait, UI rendering separate
2. **Use command pattern for undo** - Either via `undo` crate or snapshot-based
3. **Implement type checking early** - Define `DataType` enum with compatibility rules
4. **Cache aggressively** - Node shapes, wire meshes, evaluation results
5. **Use spatial indexing** - R-tree for large graphs with many nodes
6. **LOD for zoom** - Simplify rendering at low zoom levels
7. **Serialize with versioning** - Plan for schema evolution from day one
