# Arc Documentation Progress

**Status:** Living document  
**Last updated:** 2026-09-11  

This document tracks the authoring, review, and acceptance status of the Arc documentation suite.

---

## Status Legend

| Marker | Meaning |
|---|---|
| ✅ | Complete — reviewed and accepted |
| 📝 | Drafted — content written, under active review |
| 📋 | Stub — outline exists, content pending |
| ❌ | Missing — planned but not yet started |

---

## Progress Overview

| Document | Category | Status | Notes |
|---|---|---|---|
| [`architecture.md`](architecture.md) | Vision & Foundation | ✅ Complete | Master architectural vision and visual design manifesto |
| [`requirements.md`](requirements.md) | Requirements | ✅ Complete | Traceable requirement specifications (`REQ-*`) |
| [`glossary.md`](glossary.md) | Terminology | ✅ Complete | Canonical Arc domain concepts |
| [`decisions/0001-wayland-compositor-as-operating-substrate.md`](decisions/0001-wayland-compositor-as-operating-substrate.md) | ADR | ✅ Complete | Smithay-based Wayland compositor in Rust |
| [`decisions/0002-wgpu-native-scene-graph-over-webviews.md`](decisions/0002-wgpu-native-scene-graph-over-webviews.md) | ADR | ✅ Complete | Elimination of WebViews; direct WGPU rendering |
| [`decisions/0003-zero-input-ambient-canvas.md`](decisions/0003-zero-input-ambient-canvas.md) | ADR | ✅ Complete | Global keyboard intent capture without input widgets |
| [`decisions/0004-dual-channel-computer-use-visual-choreography.md`](decisions/0004-dual-channel-computer-use-visual-choreography.md) | ADR | ✅ Complete | Decoupling visual animation from deterministic IPC |
| [`decisions/0005-pipewire-native-full-duplex-audio.md`](decisions/0005-pipewire-native-full-duplex-audio.md) | ADR | ✅ Complete | PipeWire audio node, Silero VAD, streaming local TTS/STT |
| [`decisions/0006-deterministic-enforcement-plane.md`](decisions/0006-deterministic-enforcement-plane.md) | ADR | ✅ Complete | Arc Deterministic Policy Broker, Guardian, and Staged Executor |
| [`decisions/0007-zero-copy-gpu-frame-capture.md`](decisions/0007-zero-copy-gpu-frame-capture.md) | ADR | ✅ Complete | DMA-BUF and Vulkan/CUDA zero-copy VRAM-to-VRAM frame ingestion |
| [`decisions/0008-native-microphone-sensory-pipeline-aec.md`](decisions/0008-native-microphone-sensory-pipeline-aec.md) | ADR | ✅ Complete | Direct ALSA ring buffers, WebRTC AEC, and low-power VAD barge-in |
| [`decisions/0009-scoped-browser-sessions-and-stealth.md`](decisions/0009-scoped-browser-sessions-and-stealth.md) | ADR | ✅ Complete | Scoped TPM2 browser session bridging, CDP stealth, and 2FA handover |
| [`decisions/0010-generative-spatial-surface-continuum.md`](decisions/0010-generative-spatial-surface-continuum.md) | ADR | ✅ Complete | Post-application OS: Engine Pipes, spatial surfaces, living bookmark wall |
| [`decisions/0011-data-provenance-taint-tracking-and-irreversible-actions.md`](decisions/0011-data-provenance-taint-tracking-and-irreversible-actions.md) | ADR | ✅ Complete | Context taint tracking, Class-I irreversible actions, cryptographic signoff |
| [`decisions/0012-generative-domain-scenes-and-kinetic-lifecycle.md`](decisions/0012-generative-domain-scenes-and-kinetic-lifecycle.md) | ADR | ✅ Complete | Validated Scene-IR, zero-allocation state binding, 3D raycast hit arbitration, two-tier checkpointing |
| [`milestones/0001-nested-smithay-kinetic-canvas-spike.md`](milestones/0001-nested-smithay-kinetic-canvas-spike.md) | Milestone | 📝 Drafted | Milestone 1: Nested Smithay spike, kinetic typography |
| [`milestones/0002-spatial-surface-engine-pipes.md`](milestones/0002-spatial-surface-engine-pipes.md) | Milestone | 📝 Drafted | Milestone 2: Generative spatial surfaces, DMA-BUF ingestion, living bookmark wall |
| [`milestones/0003-dual-channel-agency-and-deterministic-enforcement.md`](milestones/0003-dual-channel-agency-and-deterministic-enforcement.md) | Milestone | 📝 Drafted | Milestone 3: Ghost pointer, privileged socket gating, Policy Broker & Guardian |
| [`milestones/0004-pipewire-native-voice-duplex.md`](milestones/0004-pipewire-native-voice-duplex.md) | Milestone | 📝 Drafted | Milestone 4: PipeWire native node, AEC3, Silero VAD, conversational barge-in |
| [`milestones/0005-sovereign-standalone-drm-kms-boot.md`](milestones/0005-sovereign-standalone-drm-kms-boot.md) | Milestone | 📝 Drafted | Milestone 5: Direct DRM/KMS backend, libseat/udev, systemd standalone boot |

---

## Summary Statistics

```text
Design docs:   3 of 3 complete   (100%)
ADRs:          12 of 12 complete (100%)
Milestones:    5 of 5 drafted    (100%)
Total suite:   20 documents
```

---

## Document Dependency Graph

```mermaid
graph TD
    ARCH["architecture.md (Vision & Principles)"]
    REQ["requirements.md (Traceable Specs)"]
    GLOSS["glossary.md (Definitions)"]

    ADR01["ADR-0001: Wayland Compositor Substrate"]
    ADR02["ADR-0002: WGPU Native Scene Graph"]
    ADR03["ADR-0003: Zero-Input Ambient Canvas"]
    ADR04["ADR-0004: Dual-Channel Agency"]
    ADR05["ADR-0005: PipeWire Voice Duplex"]
    ADR06["ADR-0006: Deterministic Enforcement"]
    ADR07["ADR-0007: Zero-Copy GPU Frame Capture"]
    ADR08["ADR-0008: Native Mic Pipeline & AEC"]
    ADR09["ADR-0009: Scoped Browser Sessions & Stealth"]
    ADR10["ADR-0010: Generative Spatial Surface Continuum"]
    ADR11["ADR-0011: Context Taint & Class-I Mutations"]
    ADR12["ADR-0012: Domain Scenes & Kinetic Lifecycle"]

    M01["Milestone 0001: Nested Canvas Spike"]
    M02["Milestone 0002: Spatial Surfaces & Engine Pipes"]
    M03["Milestone 0003: Dual-Channel Agency & Broker"]
    M04["Milestone 0004: PipeWire Voice Duplex"]
    M05["Milestone 0005: Sovereign Standalone DRM Boot"]

    ARCH --> REQ
    ARCH --> GLOSS
    REQ --> ADR01
    REQ --> ADR02
    REQ --> ADR03
    REQ --> ADR04
    REQ --> ADR05
    REQ --> ADR06
    REQ --> ADR07
    REQ --> ADR08
    REQ --> ADR09
    REQ --> ADR10
    REQ --> ADR11
    REQ --> ADR12

    ADR01 & ADR02 & ADR03 --> M01
    M01 & ADR07 & ADR10 & ADR12 --> M02
    M02 & ADR04 & ADR06 & ADR09 & ADR11 --> M03
    M01 & ADR05 & ADR08 --> M04
    M01 & M02 & M03 & M04 --> M05
```

