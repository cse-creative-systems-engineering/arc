# ADR-0001: Wayland Compositor as the Operating Substrate

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Previous explorations in AI operating systems (such as Aios v0.1) relied on a desktop application model: running a Tauri or Electron app hosting a webview over an existing X11 or Wayland window manager (GNOME, KDE, Sway). 

This approach creates fundamental architectural friction:
1. **The Tenant Problem**: The AI is merely an application window competing with other windows. It is constrained by external window managers, surface layer-shell protocols, focus stealing guards, and dock fallbacks.
2. **Lack of Spatial Authority**: An application-level agent cannot natively choreograph window placement, organize 3D focal stages, or intercept global keyboard and pointer events cleanly.
3. **Overhead**: Running a webview engine incurs hundreds of megabytes of memory tax and redundant event loops completely disconnected from the underlying OS display server.

To achieve a truly integrated operating system where AI is the environment rather than a visitor, Arc must own the display server.

## Decision

**Arc will be built directly as a Wayland compositor using Smithay in Rust.**

Arc will manage the hardware display pipeline directly via Linux DRM/KMS and `libinput`. It will not run inside GNOME, KDE, or another window manager in production; it **is** the display server and window manager.

During development, Arc will support a nested window backend (`smithay::backend::winit`) so developers can iterate inside an existing desktop session without reboots.

## Consequences

**Positive:**
- Arc possesses absolute authority over window placement, z-ordering, rendering pipelines, and input routing.
- Eliminates the layer-shell negotiation bugs, EWMH dock fallbacks, and click-through routing hacks that plagued previous webview approaches.
- Enables native 120Hz/240Hz hardware-accelerated rendering directly to the framebuffer.
- Smithay provides battle-tested, modular Rust implementations for Wayland core protocols (xdg-shell, wl_seat, dmabuf).

**Negative:**
- Implementing a display server requires handling low-level protocol details (clipboard synchronization, seat management, fractional scaling, XWayland compatibility).
- Development complexity is higher than writing a high-level UI application.

**Neutral:**
- Standard Linux desktop applications (Firefox, Alacritty, Steam) run seamlessly on Arc as standard Wayland or XWayland clients.

## Related
- `architecture.md` §2.4, §3
- `requirements.md` REQ-BOOT-001
- `decisions/0002-wgpu-native-scene-graph-over-webviews.md`
