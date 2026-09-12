# ADR-0002: WGPU Native Scene Graph Over WebViews

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

In early generative UI prototypes, language models authored self-contained HTML/CSS fragments rendered inside webviews. While this offered initial visual flexibility, it introduced severe systems-level drawbacks:
1. **DOM Overhead & Reconciliation Latency**: Initializing and rendering webview frames requires parsing DOM trees, loading layout engines, and running JavaScript reconciliation.
2. **Disconnected Evidence Binding**: Live system state telemetry (CPU, GPU, memory, process graphs) had to be serialized across a JSON-RPC / IPC boundary into the webview, creating lag and stale frames.
3. **Aesthetic Incoherence**: Webview styling frequently clashed with native hardware acceleration, sub-pixel font rendering, and display refresh rates.

To create an operating environment that feels fluid, cinematic, and instantaneous, Arc requires a native rendering engine.

## Decision

**Arc will eliminate WebViews for core UI and render all system surfaces, typography, shaders, and window stages directly via WGPU.**

The Arc compositor will maintain a hardware-accelerated scene graph rendered with WGPU. UI elements (kinetic typography, cards, waveforms, diagnostic graphs) will be rendered as native vector geometry or GPU shaders. Models generating user interfaces will output a typed declarative Scene Graph Intermediate Representation (IR) rather than raw HTML/CSS.

Text rendering will use native GPU glyph pipelines (`cosmic-text` and `glyphon` / `vello`).

## Consequences

**Positive:**
- 0ms webview engine initialization latency.
- Instant, zero-copy data binding: scene graph nodes bind directly to memory addresses in Arc's `SystemStateStore`.
- Enables cinematic graphical effects (procedural breathing shaders, true 3D perspective, depth-of-field rack focus) directly on the GPU.
- Massive memory reduction: saves 200MB+ of browser engine memory per surface.

**Negative:**
- Generating native scene graph IR requires a more constrained schema than arbitrary web HTML/CSS.
- Rich web content (e.g., rendering actual websites) still requires spawning an external browser client (such as Chromium or Servo) inside a client window.

**Neutral:**
- When external websites must be displayed or navigated, they will be hosted in standard Wayland client windows managed on the 2.5D stage.

## Related
- `architecture.md` §3.1, §3.3
- `requirements.md` REQ-UX-002, REQ-UX-004
- `decisions/0001-wayland-compositor-as-operating-substrate.md`
