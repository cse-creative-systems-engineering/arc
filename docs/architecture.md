# Arc Architecture

**Status:** Vision and Architectural Specification  
**Depends on:** `glossary.md`, `requirements.md`, all ADRs  

> *"Give me a lever long enough and a fulcrum on which to place it, and I shall move the world."*  
> — Archimedes of Syracuse

**Arc** is an intent-first, autonomous operating substrate. It is neither a conversational wrapper nor an application running inside an existing desktop environment. Arc **is** the operating system interface: a custom Wayland display server, scene graph, and execution runtime built in Rust on modern Linux primitives.

---

## 1. The Core Philosophy

### The Death of the 1973 Metaphor
For more than fifty years—from the Xerox Alto to macOS Sonoma and Windows 11—personal computing has been trapped inside the **WIMP** paradigm: Windows, Icons, Menus, Pointer. 

Under WIMP, the operating system is passive and dumb:
* To perform a task, the human must manually locate applications, traverse filesystem hierarchies, manage overlapping rectangular viewports, click inside input boxes, and manually shuttle data between programs.
* When something breaks, the OS writes cryptic logs to disk and waits for the human to notice.
* When AI is added, it is invariably bolted on as a tenant—a chat window floating over the desktop or a browser sidebar fighting with the window manager for screen space.

### The Archimedes Lever
Archimedes recognized that mechanical advantage transforms human effort. Arc applies this principle to computing:

```text
Human Intent (Will)  ──►  Arc (The Lever)  ──►  Linux Kernel & Subsystems (The Fulcrum)  ──►  Reality Moved
```

In Arc:
- The human expresses intent through natural language (voice or typing).
- Arc decomposes, plans, verifies, stages, and visibly executes the required transformations.
- The machine remains tranquil until called upon, and operates with transparent, visible choreography when active.

---

## 2. System Architecture: The Four Planes

Arc is divided into four distinct planes of responsibility:

```
┌────────────────────────────────────────────────────────────────────────┐
│                              HUMAN                                     │
│               Voice (Duplex)  │  Keystrokes (Zero-Input)               │
└───────────────────────────────┼────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│                           SENSORY PLANE                                │
│   PipeWire Audio Graph (VAD)  │  libinput Event Stream                 │
│   AT-SPI Accessibility Tree   │  eBPF Kernel Ring Buffers              │
└───────────────────────────────┼────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       INTENT & REASONING PLANE                         │
│   Tier-0 Reflex Engine (Local SLM, instant parsing & stream echo)      │
│   Tier-1 Cognitive Planner (Decomposition & multi-step execution)      │
│   Tier-2 Verification Role (Independent challenge & safety audit)      │
└───────────────────────────────┼────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     ENFORCEMENT PLANE (Deterministic)                  │
│   Policy Broker (Capability × Clearance matrix)                        │
│   Infrastructure Guardian (Invariant vetoes)                           │
│   Staged Transaction Executor (Checkpoint → Stage → Health → Commit)   │
│   TPM2 Credential Vault & Scoped Browser Identity                      │
└───────────────────────────────┼────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        DISPLAY & CANVAS PLANE                          │
│   Smithay Wayland Compositor  │  WGPU Hardware Scene Graph             │
│   2.5D Amphitheater Stage     │  Kinetic Typography Engine             │
│   Ghost Pointer Choreography  │  Procedural Ambient Shaders            │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 The Sensory Plane
The Sensory Plane continuously monitors physical inputs and kernel telemetry:
* **Audio**: Connects as a dedicated filter node in the **PipeWire** pro-audio graph. Runs zero-overhead Voice Activity Detection (Silero VAD) to detect speech onset and completion.
* **Input**: Hooks raw `libinput` streams. When no client window holds modal focus, all keyboard and gesture events flow into the ambient canvas.
* **Semantic Context**: Reads application state via the Wayland accessibility protocols and Linux AT-SPI, avoiding clumsy OCR.
* **Kernel Telemetry**: Uses eBPF probes on network sockets, process schedulers, and block I/O to maintain continuous awareness of hardware health.

### 2.2 The Intent & Reasoning Plane
Arc employs a tiered cognitive hierarchy:
* **Tier-0 (Spinal Reflex, < 30ms)**: Tiny local models (0.5B–1.5B or ONNX pipelines) responsible for instant keystroke intent disambiguation, syntax classification, and UI responsiveness.
* **Tier-1 (Cognitive Planner, 3B–70B or Frontier API)**: Decomposes complex user instructions (*"Set up a Qwen3.6-VL training pipeline"*) into directed acyclic graphs (DAGs) of executable actions.
* **Tier-2 (Verification Role)**: An independent reasoning pass that challenges the plan, checks failure modes, and flags potential security violations before passing execution to the enforcement plane.

### 2.3 The Enforcement Plane
Inherited directly from the battle-tested architecture of Aios:
* **Policy Broker**: Enforces two-dimensional authorization ($Capability \times Clearance$). No model output is ever executed directly as raw shell commands.
* **Infrastructure Guardian**: Deterministic safety checks that enforce non-negotiable invariants (e.g., bootloader integrity, data loss prevention, firewall rules).
* **Staged Transaction Executor**: Every consequential mutation is staged in an isolated transaction with verified health checks and automatic rollback capabilities.
* **Credential Vault**: Hardware-backed (TPM2 / encrypted storage) holding API keys, SSH identities, and persistent browser session cookies.

### 2.4 The Display & Canvas Plane
Arc controls the entire display through a custom Wayland compositor built with **Smithay** and **WGPU**:
* Bypasses X11, GNOME, KDE, and WebViews completely.
* Renders the UI directly onto the DRM/KMS framebuffer with hardware-accelerated shaders at native monitor refresh rates (120Hz/240Hz).

---

## 3. The Visual and Spatial Design

### 3.1 The Obsidian Void and Boot Sequence
Arc boots with zero visual noise:
1. **Cold Boot (< 1.5s)**: From UEFI POST, the kernel boots directly into the Arc DRM/KMS compositor without an intermediate display manager.
2. **The Procedural Breathing Watermark**: The screen is pure OLED black (`#000000`). A colossal typographic wordmark—**ARC**—fades in smoothly across the display via a procedural WGPU fragment shader, then dissolves into a subtle, low-opacity (5%) ambient watermark that breathes faintly in the background.
3. **The Welcome Prompt**: Dead-center in the middle third of the screen, the system streams its prompt in oversized, elegant display typography:
   > *Welcome to Arc. What would you like to do today?*
4. Characters appear letter-by-letter with a calibrated typewriter cadence that mimics biological phrasing rather than mechanical timers.

### 3.2 The Zero-Input Canvas
Arc features no search boxes, docks, or permanent text fields:
* **The Screen is the Buffer**: The human simply starts typing.
* Keystrokes appear instantly in the center of the canvas in razor-sharp kinetic typography.
* Backspace removes glyphs with physics-based spring transitions.
* Hitting `Enter` commits the intent to the reflex engine.

### 3.3 The 2.5D Amphitheater Stage
When windows or visual tasks are summoned, they do not clutter a flat desktop:
* **Curved Focal Plane**: Windows exist in a 3D perspective projection with depth-of-field blur.
* **Cinematic Rack Focus**: When Arc works on a window (e.g., Indeed.com), that window is in razor-sharp focus front and center. Secondary reference windows (e.g., Webmail) sit angled slightly in the periphery with a soft depth-of-field blur. When focus shifts, the camera smoothly racks focus.

---

## 4. Generative Domain Scenes

Traditional operating systems treat all stored data as hierarchical trees of inodes. Arc replaces the file manager with **Generative Domain Scenes**:

```text
User: "What's in the music directory?"
  │
  ▼
[Filesystem Query: ~/Music] ──► [Domain Classifier: Audio/Acoustic]
  │
  ▼
[Compositor Instantiates Native WGPU Scene]
  ├── Album Artwork (Extracted from metadata in 3D perspective)
  ├── Spectral Audio Visualizer (Bound to PipeWire output stream)
  └── Interactive Track Gallery (Press Space / Speak to stream)
```

- **Music / Audio**: Generates a high-fidelity listening room with vinyl/cassette artwork, track timelines, and integrated PipeWire audio playback.
- **Research / Documents**: Generates a research desk with interactive document summaries, citation trees, and concept graphs.
- **Code Repositories**: Generates an architectural dependency map with real-time test status, commit diffs, and health indicators.

---

## 5. Autonomous Systems Engineering

Arc is built for serious developers and researchers. When instructed:
> *"I want to setup an ML workflow with the new Qwen3.6 VL model. Download it and setup a workflow for training it. We'll create the dataset once it's ready."*

Arc does not merely dump instructions—it executes the workflow as an orchestrated engineering mission:

1. **Hardware & Capability Discovery**: Checks available GPU hardware (`nvidia-smi` / DRM devices), verifying VRAM capacity and CUDA toolkit compatibility.
2. **Isolated Environment Staging**: Creates a sandboxed development workspace using `uv` or isolated rootless namespaces (`bwrap`).
3. **Weight Streaming & Hub Verification**: Pulls model weights directly with chunked SHA-256 integrity verification, rendering a sleek progress gauge in a monospace stage terminal.
4. **Scaffolding & Synthetic Verification**: Generates the PyTorch/HuggingFace LoRA training harness, configures 4-bit quantization and flash-attention, and runs a synthetic forward/backward pass with dummy tensors to verify that the pipeline executes without out-of-memory (OOM) errors.
5. **Standby Handover**: Reports completion via ambient voice and kinetic typography, waiting for the human's dataset instructions.

---

## 6. Visible Agency and Dual-Channel Control

The critical failure of traditional "computer use" agents is relying on imprecise vision models to click coordinates on screenshots. Arc solves this through **Dual-Channel Control**:

```
                       Task Intent (e.g. Indeed & Email)
                                      │
                                      ▼
                        Staged Orchestration Plan
                                      │
                 ┌────────────────────┴────────────────────┐
                 ▼                                         ▼
         Control Channel                            Visual Channel
      (Deterministic IPC)                       (Human Transparency)
  - Chrome DevTools Protocol (CDP)          - Compositor spawns browser window
  - Linux AT-SPI Accessibility Tree         - Window animates onto stage
  - Exact DOM element manipulation          - Ghost cursor glides with spring easing
  - Zero misclicks / instant auth           - User watches every action in real time
```

### 6.1 The Ghost in the Machine
* **The Luminous Beacon**: When Arc operates on a window, the compositor renders a soft, luminous cursor that glides smoothly across the UI using cubic-bezier curves.
* **Tactile Feedback**: As the cursor reaches buttons or fields, the UI elements gently illuminate, dropdowns expand, and forms populate at human-comprehensible speed.
* **Narrative Pulse**: In the negative space below the active window, a single line of streaming monospace text narrates Arc's internal progress in real time.

---

## 7. Native Full-Duplex Ambient Audio

Arc treats voice not as a command trigger, but as a continuous conversational medium:

* **Direct PipeWire Node**: Arc operates as an active client in the Linux audio graph, avoiding ALSA/PulseAudio legacy latency.
* **Automatic Media Ducking**: When the user speaks, Arc smoothly attenuates running audio (music, video) by 70%, processes the speech, and restores the volume curve seamlessly upon completion.
* **Ultra-Low Latency Pipeline**:
  - Silero VAD detects speech onset within 10ms.
  - Streaming transcription (Whisper / Moonshine) pipes tokens to the reflex engine in real time.
  - Local neural text-to-speech (e.g., Kokoro-82M) synthesizes high-fidelity human speech chunks in under 30ms.
* **Conversational Personality**: Arc communicates with the calm, capable demeanor of a trusted engineering colleague—calling the user by name (*"OK, Shane..."*) and maintaining shared conversational context across visual and audio modalities.
