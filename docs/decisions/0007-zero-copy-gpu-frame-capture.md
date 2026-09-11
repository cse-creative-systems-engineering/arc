# ADR-0007: Zero-Copy GPU Frame Capture via DMA-BUF

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Conventional desktop AI agents (such as Claude Computer Use or operating system wrappers) capture visual context through userland screenshot APIs (e.g., X11 `GetImage`, Wayland screencopy portal, or `scrot`).

This approach introduces severe performance bottlenecks:
1. **PCIe Readback Latency**: Pixels must be transferred across the PCIe bus from GPU VRAM to host system RAM, taking 50–200ms and stalling the graphics pipeline.
2. **Encoding/Decoding Tax**: Images must be encoded into PNG/JPEG buffers on the CPU, transmitted over IPC, and decoded back into GPU memory as model tensors.
3. **Occlusion & Noise**: Screenshots capture the entire cluttered desktop rather than the specific application window of interest.
4. **Blind Polling**: External agents lack frame damage tracking, forcing continuous CPU polling even when the screen is static.

Because Arc is the display server, it already allocates and renders all screen content directly in GPU memory.

## Decision

**Arc will implement zero-copy frame ingestion directly within GPU VRAM using Wayland DMA-BUF handles and Vulkan/CUDA external memory interop.**

1. **VRAM-to-VRAM Tensor Ingestion**:
   - The composite framebuffer and individual client surface textures exist in physical VRAM as GBM/DMA-BUF backing objects.
   - When visual context is required by a local vision model (e.g., Qwen-VL), Arc exports the DMA-BUF file descriptor directly into the inference engine via Vulkan-CUDA External Memory (`VK_KHR_external_memory_fd`) or EGL-CUDA interop (`cudaGraphicsEGLRegisterImage`).
   - Zero pixels cross the PCIe bus to host RAM.

2. **Isolated Surface Feeds**:
   - Because client windows submit their individual surface textures to the compositor prior to composition, Arc can pass an isolated window texture (e.g., an unobstructed browser view) to the model without screen clutter or visual occlusion.

3. **In-VRAM Compute Preprocessing**:
   - A WGPU compute shader executes on the GPU texture to downsample, crop, and normalize pixel values directly into the patch layout required by the Vision Transformer in < 0.5ms.

4. **Damage-Driven Awakening**:
   - Visual inspection is triggered exclusively by Wayland damage events (`wl_surface.damage`). When the screen is static, the visual sensory pipeline consumes 0% GPU compute.

5. **VRAM Priority & OOM Crash Prevention**:
   - The compositor hardware framebuffer, swapchain, and client surface buffers maintain **Tier-0 hardware priority** in VRAM.
   - When overall VRAM utilization exceeds 85%, Arc's resource manager halts local vision tensor allocations and automatically falls back to offloading the vision weights to unified system RAM (via CPU GGUF inference) or remote gateway endpoints.
   - Under no circumstances may an inference model allocation trigger a GPU out-of-memory error, kernel panic, or display driver reset (TDR).

## Consequences

**Positive:**
- Frame ingestion latency drops from 200–500ms to < 1 millisecond.
- Eliminates CPU memory thrashing and PCIe bus saturation.
- Delivers clean, unoccluded per-window textures directly to reasoning models.
- Damage-driven triggers ensure zero idle GPU power consumption.
- Prevents desktop crashes and display resets under heavy VRAM workloads via deterministic memory tiers.

**Negative:**
- Requires GPU driver support for DMA-BUF sharing and Vulkan/CUDA external memory extensions (standard on modern Linux with Mesa/NVIDIA 550+).
- Tightly couples the compositor's rendering backend to external memory tensor formats.

## Related
- `architecture.md` §2.1, §6
- `requirements.md` REQ-AGENT-004
- `decisions/0001-wayland-compositor-as-operating-substrate.md`
- `decisions/0002-wgpu-native-scene-graph-over-webviews.md`
