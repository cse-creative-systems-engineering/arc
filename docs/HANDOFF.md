# Arc Project Handoff

**Date:** 2026-09-12  
**Repository:** `https://github.com/cse-creative-systems-engineering/arc`  
**Working Directory:** `~/Arc`  
**License:** GPL-3.0-only  

---

## 1. Executive Summary & Architecture

Arc (Archimedes) is an intent-first, post-application operating substrate replacing traditional WIMP desktops with a custom Wayland compositor written in Rust and WGPU.

The system is organized into four architectural planes:
1. **Sensory Plane:** PipeWire audio graph, Silero VAD, libinput, AT-SPI, eBPF telemetry.
2. **Intent & Reasoning Plane:** Tier-0 reflex (<30ms, local/SLM), Tier-1 cognitive planner (frontier/local LLM), Tier-2 independent verification.
3. **Enforcement Plane (Deterministic):** Policy Broker ($Capability \times Clearance$), Infrastructure Guardian (safety invariants), Staged Transaction Executor (Class-R reversible vs. Class-I irreversible actions with visual approval gates), TPM2 credential vault.
4. **Display & Canvas Plane:** Smithay Wayland compositor, WGPU 3D spatial continuum, headless Engine Pipes (Chromium, PTY) streaming DMA-BUF textures onto generative surfaces without window chrome.

---

## 2. Completed Milestones & Current Progress

### Milestone 0001: Nested Smithay Kinetic Canvas Spike (Complete)
- **Rust Workspace Scaffold:** `crates/arc-compositor` and `crates/arc-engine-supervisor`.
- **Display Substrate & Nested Backend:**
  - Standalone WGPU renderer (`arc-compositor`) with pure obsidian background, kinetic typography with per-glyph critically-damped spring dynamics (`kinetics.rs`), and zero-input keystroke capture.
  - Nested Smithay 0.7 compositor (`arc-nested`) via `backend_winit` + GLES renderer with full Wayland `Seat` state machine and `xkbcommon` keysym translation directly feeding the Global Intent Buffer.
- **Reflex Bridge:** Connected to OpenRouter using `nvidia/nemotron-3-ultra-550b-a55b:free`. System prompt is optimized for audio playback (short declarative sentences, no markdown filler) with letter-by-letter visual streaming on canvas (`reflex.rs`). Credentials live in `~/.config/arc/arc.env` (never committed).
- **Startup Pacing:** Watermark display has been suppressed (`alpha: 0.0`), bypassing opening delays to bring the prompt online immediately at startup.

### Milestone 0002: Spatial Surface Continuum & Engine Pipes (Underway)
- **Engine Supervisor Crate (`arc-engine-supervisor`):**
  - `EngineSupervisor`: Async thread-safe registry tracking active headless engines.
  - `TerminalPipe`: Headless ANSI PTY runner using `portable-pty`.
  - `WebEnginePipe`: Headless Chromium runner with hardened flags (`--headless=new`, `--use-gl=angle`, `--use-angle=vulkan`, `--remote-debugging-pipe`).
- **3D Spatial Surface Continuum (`spatial.rs`):**
  - 3D model transforms, perspective depth camera, and screen-space unprojection to 3D world rays.
  - Deterministic 3D raycast hit arbitration (`arbitrate_pointer_hit`) with rejection of transparent clickjack blockers (`REQ-SURF-004`, `REQ-SCENE-004`).
- **Scene Graph IR (`scene_ir.rs`):**
  - Schema-validated, bounded AST (`MAX_SCENE_DEPTH = 8`, `MAX_CONTAINER_CHILDREN = 64`) with fail-closed diagnostics.
- **Shader Pipeline (`shaders/spatial_surface.wgsl`):**
  - Perspective surface rendering with 1px glassmorphic card borders and depth-of-field rack focus attenuation.

---

## 3. Workspace Test Status

All 20 unit tests across the workspace pass:
```bash
cargo test --workspace
```
- `arc-compositor`: 17 tests (intent lifecycle, kinetic spring convergence, pop deactivation, reflex config parsing, spatial ray-plane intersection, raycast arbitration, transparent blocker filtering, SceneIR bounds validation).
- `arc-engine-supervisor`: 3 tests (supervisor registration, PTY terminal echo, web engine options builder).

---

## 4. Next Priorities for Incoming Agent

1. **Milestone 0002 Compositor Integration:**
   - Integrate `spatial.rs` and `spatial_surface.wgsl` into the primary render loop so spatial surfaces (e.g. `ThumbnailGrid` or mock engine quads) are drawn directly on the canvas.
   - Implement the **Living Bookmark Wall** test scene (`cargo run -p arc-compositor --bin arc-compositor -- --scene bookmark-wall`).
   - Connect `arc-engine-supervisor` PTY output and Web engine frames into GPU texture bindings.
2. **DMA-BUF Zero-Copy Ingestion:**
   - Ingest client offscreen DMA-BUF handles into WGPU textures via Vulkan external memory (`VK_KHR_external_memory_fd`) and Smithay buffer import abstractions.
3. **Milestone 0003 Preparations:**
   - Design the `PolicyBroker` state machine and `InfrastructureGuardian` invariant checkers in an `arc-enforcement` crate.
