# Arc Glossary

**Status:** Living Terminology Reference  
**Depends on:** `architecture.md`  

This document provides canonical definitions for the concepts, mechanisms, and components comprising the **Arc** operating substrate.

---

## A

### Ambient Canvas
The root display plane of the Arc Wayland compositor. Rendered on pure black (`#000000`) with a subtle, breathing procedural shader watermark, the ambient canvas contains no fixed desktop icons, docks, or permanent window frames.

### Audio Ducking
The automated process of attenuating background audio streams (music, video, notifications) through PipeWire by a calibrated ratio (70%) whenever human speech is detected by Voice Activity Detection.

---

## C

### Cognitive Planner (Tier-1)
The deliberative reasoning layer responsible for decomposing high-level user instructions into directed acyclic graphs (DAGs) of discrete system and browser actions.

---

## D

### Dual-Channel Control
The architectural separation of autonomous interaction into two synchronized channels:
1. **Control Channel**: Deterministic, error-free API and IPC manipulation (CDP, AT-SPI, system calls).
2. **Visual Channel**: Physical mouse and keyboard choreography rendered on screen (Wayland virtual pointer, glowing cursor) so the human can observe the work in real time.

---

## E

### Engine Pipe
A headless background execution process (e.g., headless Chromium, PTY shell subprocess, PipeWire audio stream) that renders content directly into an offscreen GPU DMA-BUF texture without instantiating application window chrome, borders, or toolbars.

---

## G

### Generative Domain Scene
An interactive, GPU-rendered visual environment instantiated on demand in response to a filesystem query. Replaces traditional directory listings (`ls`) with domain-tailored interfaces (e.g., Acoustic Listening Room, Codebase Architecture Landscape).

### Generative Spatial Surface
A dynamically synthesized visual surface rendered directly into the compositor's WGPU scene graph. Ingests raw textures from engine pipes and renders contextual controls, cards, or viewports without application window boundaries.

### Ghost Pointer (Ghost in the Machine)
The luminous cursor rendered by the Arc compositor to visualize autonomous system interactions. Moves with biological cubic-bezier easing and illuminates UI targets before interaction.

### Global Intent Buffer
The compositor-level input buffer that captures raw keyboard keystrokes from `libinput` whenever no interactive client surface holds active modal focus.

---

## K

### Kinetic Typography
GPU-accelerated vector text rendering that streams characters with biological typewriter cadence and physical spring transitions during deletion and completion.

---

## P

### PipeWire Node
A native client connection within Linux's PipeWire pro-audio graph, enabling low-latency, bi-directional audio capture and playback with direct access to audio stream mixer nodes.

### Policy Broker
Arc's deterministic authorization gatekeeper that evaluates every proposed action against a strict capability $\times$ clearance matrix before execution.

---

## R

### Rack Focus
The visual camera transition in the 3D perspective canvas that shifts optical sharpness to the currently active spatial surface while smoothly applying a depth-of-field blur to surrounding background surfaces.

### Reflex Engine (Tier-0)
The ultra-low-latency (< 30ms) local inference component responsible for keystroke classification, prompt stream echoing, and instant UI state transitions.

---

## S

### Spatial Layout Synthesis
The capability of the Arc compositor to dynamically construct, shape, position, and group visual surfaces on the canvas in response to natural language intent (e.g., creating a living bookmark wall, an ambient audio monitor, or an ML progress desk).

### Spatial Surface Continuum
The post-application desktop architecture in Arc. All visual entities exist as fluid, borderless surfaces living directly within a single continuous GPU canvas, eliminating application window frames, title bars, and standalone window managers.

### Staged Transaction Executor
The deterministic execution engine that wraps all filesystem, package, network, and system mutations in isolated transactions featuring pre-flight checks, health verification, and automatic rollback.

---

## Z

### Zero-Input
The interaction paradigm in which the screen itself acts as the input surface. Eliminates search bars, prompt inputs, and focus-acquisition clicks by routing all ambient keystrokes directly to the intent engine.
