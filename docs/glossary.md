# Arc Glossary

**Status:** Living Terminology Reference  
**Depends on:** `architecture.md`  

This document provides canonical definitions for the concepts, mechanisms, and components comprising the **Arc** operating substrate.

---

## A

### Ambient Canvas
The root display plane of the Arc Wayland compositor. Rendered on pure black (`#000000`) with a subtle, breathing procedural shader watermark, the ambient canvas contains no fixed desktop icons, docks, or permanent window frames.

### Amphitheater Stage (2.5D Stage)
The spatial window layout engine within the Arc compositor. Windows are arranged along a curved 3D focal arc with depth-of-field perspective, allowing the system to rack focus between active and background applications.

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

## G

### Generative Domain Scene
An interactive, GPU-rendered visual environment instantiated on demand in response to a filesystem query. Replaces traditional directory listings (`ls`) with domain-tailored interfaces (e.g., Acoustic Listening Room, Codebase Architecture Landscape).

### Ghost Pointer (Ghost in the Machine)
The luminous cursor rendered by the Arc compositor to visualize autonomous system interactions. Moves with biological cubic-bezier easing and illuminates UI targets before interaction.

### Global Intent Buffer
The compositor-level input buffer that captures raw keyboard keystrokes from `libinput` whenever no client application window holds active modal focus.

---

## K

### Kinetic Typography
GPU-accelerated vector text rendering that streams characters with biological typewriter cadence and physical spring transitions during deletion and completion.

---

## P

### PipeWire Node
A native client connection within Linux's PipeWire pro-audio graph, enabling low-latency, bi-directional audio capture and playback with direct access to audio stream mixer nodes.

### Policy Broker
The deterministic authorization gatekeeper inherited from Aios that evaluates every proposed action against a strict capability $\times$ clearance matrix before execution.

---

## R

### Rack Focus
The visual camera transition in the 2.5D amphitheater stage that shifts optical sharpness to the currently active application window while smoothly applying a depth-of-field blur to surrounding windows.

### Reflex Engine (Tier-0)
The ultra-low-latency (< 30ms) local inference component responsible for keystroke classification, prompt stream echoing, and instant UI state transitions.

---

## S

### Staged Transaction Executor
The deterministic execution engine that wraps all filesystem, package, network, and system mutations in isolated transactions featuring pre-flight checks, health verification, and automatic rollback.

---

## Z

### Zero-Input
The interaction paradigm in which the screen itself acts as the input surface. Eliminates search bars, prompt inputs, and focus-acquisition clicks by routing all ambient keystrokes directly to the intent engine.
