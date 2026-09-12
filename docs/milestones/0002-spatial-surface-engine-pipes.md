# Milestone 0002: Generative Spatial Surfaces & Engine Pipes

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `requirements.md`, ADR-0002, ADR-0007, ADR-0010, ADR-0012, Milestone 0001  

## Objective

Eradicate traditional application windows and title bars by establishing the **Spatial Surface Continuum**. This milestone implements headless Engine Pipes that render web and terminal content directly into offscreen GPU DMA-BUF textures, maps them onto a curved 3D perspective WGPU canvas, enables deterministic 3D raycast input arbitration, and validates the canonical **Living Bookmark Wall** proof-of-concept.

---

## Scope and Deliverables

1. **Headless Engine Pipe Subsystem**:
   - Create the `arc-engine-supervisor` module to manage spawned headless processes.
   - Implement headless Chromium runner (`--headless=new`, `--use-gl=angle`, `--use-angle=vulkan`, `--remote-debugging-pipe`).
   - Implement headless PTY terminal pipe using `portable-pty`, rendering ANSI output directly to texture quads.

2. **Zero-Copy DMA-BUF Texture Ingestion**:
   - Ingest client offscreen DMA-BUF handles into WGPU textures using Vulkan external memory (`VK_KHR_external_memory_fd`) and Smithay buffer import abstractions.
   - Handle client frame synchronization via `wl_surface.frame` callbacks at native monitor refresh rates (120Hz/144Hz).

3. **3D Kinetic Perspective Surface Canvas**:
   - Implement planar surface transformations in WGPU vertex shaders (position, pitch, yaw, depth, scale).
   - Implement dynamic camera projection and Depth-of-Field (DoF) post-processing:
     - Foreground active surface in razor-sharp focus.
     - Periphery/reference surfaces softly blurred and angled along the focal arc.

4. **Deterministic 3D Raycasting & Input Arbitration**:
   - Unproject screen-space cursor coordinates $(X, Y)$ into 3D world rays through the active camera frustum.
   - Calculate exact ray-plane intersections against surface transform bounding boxes.
   - Foremost surface ($Z_{\text{depth}} > 0$) receives exclusive pointer and touch events.
   - Occluded surfaces reject input deterministically; 2-pixel hysteresis prevents boundary jitter.

5. **The Living Bookmark Wall (Canonical Scenario)**:
   - Compile natural intent into `ArcSceneIR` specifying a `GridLayout` of bookmark thumbnail surfaces.
   - Stream live headless page captures into thumbnail slots.
   - Upon click, the thumbnail fluidly morphs and expands in-place into an active interactive web surface with zero window chrome.

---

## Acceptance Criteria

### Vignette 1: Headless Engine Pipe to 3D Canvas
- **Action**: Spawn a headless terminal engine running `btop` and a headless Chromium engine loading `https://news.ycombinator.com`.
- **Expected Result**: Both viewports appear as borderless glassmorphic spatial surfaces on the 3D canvas. Frames update smoothly with zero title bars, window frames, or desktop docks.

### Vignette 2: 3D Raycasting & Rack Focus
- **Action**: Move pointer between overlapping spatial surfaces and click the background surface.
- **Expected Result**: Camera smoothly transitions focus (rack focus) to the clicked surface, bringing it front and center while the prior surface blurs into the background. Pointer events route exclusively to the active surface.

### Vignette 3: Living Bookmark Wall Expansion
- **Action**: Click a bookmark thumbnail on the living bookmark wall.
- **Expected Result**: The thumbnail card expands across the canvas with spring physics into a full interactive web surface in-place. Keystrokes and clicks immediately interact with the web content.

---

## Verification Plan

```bash
# 1. Verify engine supervisor unit tests
cargo test -p arc-engine-supervisor

# 2. Run DMA-BUF texture ingestion tests
cargo test -p arc-compositor --test dmabuf_import

# 3. Launch spatial canvas test harness with mock engine pipes
cargo run --bin arc-compositor -- --scene bookmark-wall
```
