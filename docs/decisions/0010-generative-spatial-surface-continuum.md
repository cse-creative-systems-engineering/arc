# ADR-0010: Generative Spatial Surface Continuum (The Post-Application OS)

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

For over four decades, personal computing has been governed by the application window abstraction:
1. **The Application Monolith**: To browse the web, write code, or listen to music, the operating system launches a discrete third-party program (Chrome, Alacritty, Spotify). That program owns an isolated rectangular viewport bounded by window frames, title bars, and close buttons.
2. **The Fragmented Desktop**: The human must manually position, resize, tile, minimize, and shuffle overlapping application rectangles.
3. **Rigid Application Chrome**: Each program dictates its own user interface scaffolding (tabs, URL bars, menus, sidebars), preventing the user from customizing or unifying their spatial workspace.
4. **The False Dichotomy of "OS vs. Apps"**: The operating system acts merely as a dumb landlord renting screen real estate to foreign processes.

When designing Arc, treating modern capabilities as "opening browser windows" or "tiling apps" represents a severe regression to the 1984 WIMP paradigm.

## Decision

**Arc abolishes the concept of standalone "applications" and "window frames." The entire display is a single, continuous, GPU-accelerated spatial canvas where all visual and interactive entities exist as Generative Surfaces.**

1. **Engine Pipes Over Application Windows**:
   - Traditional programs are replaced by headless **Engine Pipes** (e.g., headless Chromium/Servo for web rendering, headless PTY for shell execution, PipeWire for audio, FFmpeg for video).
   - Engine pipes do not render window borders or UI chrome; they render directly into offscreen GPU buffers (DMA-BUFs).

2. **Compositor-Synthesized Surfaces**:
   - The Arc WGPU compositor ingests these raw GPU textures and maps them onto dynamically synthesized spatial surfaces (cards, panels, ambient backdrops, or full-stage viewports).
   - Arc generates all visual frames, controls, and surrounding context on the fly based on natural user intent.

3. **Intent-Driven Spatial Layout Synthesis**:
   - The user can direct the spatial structure of their canvas dynamically. For example:
     > *"Arc, set a portion of the desktop dedicated to holding my bookmarked sites. Display a thumbnail for each bookmark so when I click on it that page is displayed."*
   - Arc synthesizes a dedicated glassmorphic surface region, streams live headless web thumbnails into texture slots, and fluidly expands any thumbnail into a full interactive web surface in-place upon interaction.

4. **Fluid Morphing and Spatial Continuity**:
   - Surfaces do not "pop open" or "minimize." They smoothly morph, expand, dock, or dissolve across the canvas using GPU compute shaders and physical spring dynamics.

## Consequences

**Positive:**
- Complete eradication of visual clutter: no title bars, no window borders, no redundant browser chrome or docks.
- Infinite spatial malleability: the user can reshape, group, and summon visual surfaces exactly as they imagine them.
- Drastic resource efficiency: headless engine pipes share GPU buffers with zero window manager overhead.
- True unified aesthetic: every element on screen shares Arc's cinematic, typography-first visual language.

**Negative:**
- Legacy desktop applications that rely heavily on their own complex window toolbars (e.g., legacy GIMP, Blender) require specialized container surfaces or fallback XWayland presentation layers until native headless pipelines are available.

## Related
- `architecture.md` §1, §2.4, §3, §4
- `requirements.md` REQ-UX-004, REQ-SURF-001, REQ-SURF-002, REQ-SURF-003
- `decisions/0001-wayland-compositor-as-operating-substrate.md`
- `decisions/0002-wgpu-native-scene-graph-over-webviews.md`
