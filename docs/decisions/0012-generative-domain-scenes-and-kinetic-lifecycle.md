# ADR-0012: Generative Domain Scenes, Scene Graph IR & Kinetic Spatial Lifecycle

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

In Arc, the user does not open static application windows; instead, user intent dynamically synthesizes generative domain scenes on a 3D kinetic canvas (e.g., Living Bookmark Walls, Ambient Code Workbenches, Diagnostic Telemetry HUDs). 

While ADR-0010 established the post-application spatial continuum, relying on probabilistic AI models to dynamically generate visual interfaces introduces severe flight-critical vulnerabilities and performance bottlenecks:
1. **Malicious or Malformed Layout Denial-of-Service**: Unchecked scene generation can produce deeply recursive tree hierarchies, infinite layout loops, or millions of vector control points that stall the GPU rasterizer or overflow the compositor stack.
2. **Spatial Clickjacking & Input Ambiguity**: Unregulated spatial positioning could allow transparent surfaces or occluded background engine pipes to capture physical clicks or keystrokes, leading to unintended commands.
3. **Render Thread Starvation**: Binding high-frequency system telemetry (60Hz audio waveforms, build logs, kernel metrics) to canvas visual primitives via naive event dispatch or heap allocations introduces micro-stutters and drops compositor frames below the 120Hz budget.
4. **Engine Pipe State Desynchronization Across Reboots**: Spatial surfaces correspond to active headless processes. Serializing only visual coordinates leaves dead, broken surfaces on reboot, while eagerly respawning heavy browser engines simultaneously causes boot-time VRAM spikes.

## Decision

**Arc shall compile generative domain scenes through a deterministic, typed Scene Graph Intermediate Representation (`ArcSceneIR`) with zero-allocation state binding, raycast-validated hit arbitration, and two-tier session checkpointing.**

### 1. Strictly Validated Scene Graph Intermediate Representation (`ArcSceneIR`)
All generative domain scenes synthesized by Tier-0 reflex or Tier-1 planner models must compile into a strictly typed, schema-validated AST:
- **Primitive Set**: The vocabulary is strictly bounded to `SurfaceContainer`, `GridLayout`, `ThumbnailGrid`, `KineticStream`, `VectorCard`, and `DataStreamNode`.
- **Structural Constraints**:
  - Maximum AST tree depth: 8 levels.
  - Maximum child nodes per container: 64.
  - Transparent interaction blockers (opacity = 0 with hit-test enabled) are strictly rejected.
- **Fail-Closed Fallback**: If an emitted scene graph fails schema validation or constraint checks, the compositor rejects the AST entirely and renders a safe, single-node kinetic text card explaining the synthesis error without disrupting the canvas.

### 2. Zero-Allocation Lock-Free State Binding
To bind real-time operating metrics and engine pipe events to visual vector nodes:
- **Lock-Free Channels**: Telemetry streams into `arc_state_store` via bounded lock-free ring buffers (`crossbeam-channel` / atomic scalar slots).
- **Direct Staging Buffers**: High-frequency telemetry (e.g., PipeWire FFT magnitudes, PTY character deltas) updates mapped GPU uniform buffers and instanced vertex arrays via `wgpu::Queue::write_buffer_staged`.
- **Zero Allocations on Render Path**: The compositor render thread performs zero heap allocations during scene graph traversal and frame drawing.

### 3. Deterministic 3D Raycasting & Input Arbitration
To ensure seamless and unambiguous interaction in a 3D spatial canvas:
- **Ray-Plane Unprojection**: Physical and virtual pointer coordinates $(X_{\text{screen}}, Y_{\text{screen}})$ are cast as 3D rays from the virtual camera through the view frustum to intersect planar surface transforms.
- **Occlusion Masking & Focus**: The foremost surface (lowest positive $Z_{\text{depth}}$ along the ray) receives exclusive pointer capture. Occluded background surfaces reject pointer events deterministically.
- **Boundary Hysteresis**: A 2-pixel spatial hysteresis boundary prevents rapid focus oscillation during camera pans or spring-physics surface transitions.

### 4. Two-Tier Spatial Session Checkpointing
To guarantee rapid, seamless restoration across system reboots:
- **Tier-1 Spatial Ledger**: Canvas state (camera translation, rotation, zoom, surface transforms, semantic workspace clusters) is written atomically to an embedded SQLite database (`~/.local/share/arc/canvas_state.db`) on any layout modification.
- **Tier-2 Engine Session Bridging**:
  - Headless Chromium engine pipes persist session cookies, local storage, and tab history in scoped disk directories (`~/.local/share/arc/engines/web/<id>`).
  - Terminal engine pipes record ring-buffered scrollback logs.
  - On system boot, Arc renders the spatial canvas instantly using cached DMA-BUF thumbnail textures, lazily spinning up headless engine pipes only as surfaces enter the active camera frustum.

## Consequences

**Positive:**
- Guaranteed display stability: Compositor maintains 120Hz/144Hz framerates with zero heap allocations on the hot path.
- Immune to layout injection attacks, visual clickjacking, and GPU rasterizer DOS.
- Flawless spatial memory: The user's dynamic workspace arrangements persist across reboots with instantaneous visual rehydration.
- Deterministic pointer routing in 3D perspective space.

**Negative:**
- Scene graph creativity is bounded to the primitives provided in `ArcSceneIR`.
- Complex web engine pipes rehydrated on boot may require a brief lazy load period before becoming fully interactive.

## Related
- `architecture.md` §2.4, §3, §4
- `requirements.md` REQ-SURF-004, REQ-SURF-005, REQ-SCENE-001, REQ-SCENE-002, REQ-SCENE-003, REQ-SCENE-004
- `decisions/0002-wgpu-native-scene-graph-over-webviews.md`
- `decisions/0010-generative-spatial-surface-continuum.md`
