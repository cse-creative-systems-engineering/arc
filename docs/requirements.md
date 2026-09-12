# Arc Traceable Requirements

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `glossary.md`  

This document formalizes the functional, technical, safety, and experiential requirements for Arc. Every requirement has a unique, traceable identifier formatted as `REQ-<CATEGORY>-<NUMBER>`.

---

## Category Codes

| Code | Category |
|---|---|
| `BOOT` | Cold boot, DRM/KMS framebuffer, startup latency |
| `UX` | Zero-input canvas, kinetic typography, spatial surface continuum |
| `SURF` | Generative spatial surfaces, engine pipes, dynamic layout synthesis |
| `SCENE` | Generative domain scenes and semantic data representation |
| `AGENT` | Autonomous engineering, dual-channel agency, ghost choreography |
| `AUDIO` | PipeWire integration, full-duplex voice, media ducking |
| `SAF` | Policy broker, guardian invariants, staged execution, credential security |

---

## 1. Boot and Framebuffer (`REQ-BOOT`)

### REQ-BOOT-001: Sub-2-Second Direct DRM Boot
The system shall boot from UEFI POST directly into the Arc Wayland compositor via DRM/KMS without an intermediate display manager (GDM/LightDM) in less than 1.5 seconds on baseline NVMe hardware when running an Arc Unified Kernel Image (UKI with built-in DRM modules). On generic distribution kernels with modular initramfs, cold boot to DRM/KMS shall not exceed 3.0 seconds.  
*Source: Architecture §3.1*

### REQ-BOOT-002: Procedural Watermark Shader
Upon display initialization, the compositor shall execute a procedural WGPU fragment shader that fades in the typographic wordmark "ARC" across the display and smoothly decays to a persistent ambient watermark with an alpha opacity not exceeding 5%.  
*Source: Architecture §3.1*

### REQ-BOOT-003: Pure Obsidian Background
The default compositor background color shall be pure OLED black (`#000000`) with zero static desktop UI elements, icons, docks, or status bars.  
*Source: Architecture §3.1*

---

## 2. User Experience and Kinetic Canvas (`REQ-UX`)

### REQ-UX-001: Zero-Input Global Keystroke Capture & Focus Arbitration
When no application window holds active modal focus, raw keyboard input from `libinput` shall be routed directly to the compositor's global intent buffer. When a client window holds focus, keystrokes route exclusively to that client. The compositor shall support an Ambient Summon Chord (`Super` or double-tap `Escape`) that immediately reclaims keyboard focus to the Global Intent Buffer and dims client windows by 30%. Requests to inhibit global shortcuts shall be denied by default.  
*Source: Architecture §3.2, ADR-0003*

### REQ-UX-002: Real-Time Kinetic Typography
All text streamed from the intent engine or typed by the user shall be rendered directly as GPU vector glyphs using biological typewriter cadence and spring physics on character deletion.  
*Source: Architecture §3.1, §3.2*

### REQ-UX-003: Dual-Typeface Aesthetic Hierarchy
The compositor typography engine shall enforce a strict dual-typeface hierarchy: an elegant literary/editorial display face for human narrative and intent prompts, and an ultra-precise monospaced font for technical output, code diffs, and execution telemetry.  
*Source: Architecture §3.1*

### REQ-UX-004: Spatial Surface Continuum
All visual surfaces shall be rendered along a curved 3D perspective depth with GPU-accelerated depth-of-field blur. The active focus surface shall remain in sharp focus while inactive reference surfaces are softly blurred and angled in the periphery. Window title bars, maximize/minimize buttons, and window frames are abolished.  
*Source: Architecture §3.3, ADR-0010*

### REQ-UX-005: Native Input Method (IME) Integration
The compositor Global Intent Buffer shall implement the server-side Wayland `zwp_text_input_v3` protocol, enabling native input method engines (e.g., `fcitx5`, `ibus`) to display CJK pre-edit strings and candidate menus directly above the kinetic typography stream.  
*Source: ADR-0003*

---

## 3. Generative Spatial Surfaces (`REQ-SURF`)

### REQ-SURF-001: Headless Engine Texture Pipes via DMA-BUF
All non-native visual workloads (web content, terminal emulators, video streams) shall be executed by headless engine pipes (e.g., headless Chromium, PTY subprocesses) rendering directly into offscreen GPU DMA-BUF textures. Engine pipes shall not instantiate window frames or client-side UI decorations.  
*Source: Architecture §2.4, ADR-0010*

### REQ-SURF-002: Intent-Driven Spatial Layout Synthesis
The compositor shall synthesize, position, and shape surface layouts on demand in response to natural user prompts (e.g., dedicating a canvas region for live bookmark thumbnails or placing an audio monitor). Start menus, application launchers, and fixed desktop docks are abolished.  
*Source: Architecture §3.3, ADR-0010*

### REQ-SURF-003: Fluid Morphing & In-Place Surface Expansion
Surfaces shall transition fluidly across the canvas using GPU compute shaders and physical spring dynamics. Interacting with a preview or thumbnail surface (e.g., a bookmarked site card) shall expand the surface in-place into an active interactive viewport without opening a separate window entity.  
*Source: Architecture §3.3, ADR-0010*

---

## 4. Generative Domain Scenes (`REQ-SCENE`)

### REQ-SCENE-001: Abolition of Raw File Trees
Filesystem queries and directory inspections shall not be presented to the user as plain text lists or generic icon grids. The compositor shall instantiate a domain-specific visual scene corresponding to the semantic classification of the directory contents.  
*Source: Architecture §4*

### REQ-SCENE-002: Acoustic Listening Room Scene
When inspecting directories classified as audio/music, Arc shall instantiate an acoustic listening room scene displaying high-resolution album artwork, an active PipeWire spectrum analyzer, and integrated single-keystroke playback controls.  
*Source: Architecture §4*

### REQ-SCENE-003: Codebase Landscape Scene
When inspecting software projects, Arc shall render an interactive dependency and architectural scene indicating module health, git status, and test execution status.  
*Source: Architecture §4*

---

## 5. Autonomous Agency and Choreography (`REQ-AGENT`)

### REQ-AGENT-001: Dual-Channel Control Architecture & Privileged Input Gating
All autonomous web and application interactions shall be executed through deterministic control channels (CDP, AT-SPI) while simultaneously choreographing visible pointer and keyboard movements on the screen. Coordinate-guessing screenshot models shall not be used for critical path actions. Wayland virtual input protocols (`zwp_virtual_pointer_v1`, `zwp_virtual_keyboard_v1`) shall be restricted via `SO_PEERCRED` socket credentials strictly to authenticated internal Arc engine threads, terminating unauthenticated third-party client bindings immediately.  
*Source: Architecture §6, ADR-0004*

### REQ-AGENT-002: Visible Ghost Pointer Choreography
Autonomous interactions on client windows shall be visualized via a luminous ghost cursor that moves with natural cubic-bezier physics, illuminates targeted UI elements, and simulates keystrokes at human-comprehensible speeds.  
*Source: Architecture §6.1*

### REQ-AGENT-003: Autonomous Systems Engineering Pipeline
When tasked with complex engineering goals (e.g., machine learning environment setup), Arc shall autonomously discover hardware capabilities, scaffold sandboxed environments, pull model weights with integrity checks, and execute synthetic verification passes prior to declaring readiness.  
*Source: Architecture §5*

### REQ-AGENT-004: Zero-Copy GPU Frame Ingestion
The compositor shall provide direct GPU VRAM access to local vision models using Wayland DMA-BUF handles and Vulkan/CUDA external memory interop, achieving frame ingestion latency of less than 1 millisecond without PCIe bus transfers or CPU readbacks.  
*Source: Architecture §2.1, ADR-0007*

### REQ-AGENT-005: GPU VRAM Allocation Priority & OOM Crash Prevention
The compositor framebuffer, swapchain, and client surface textures shall maintain Tier-0 hardware allocation priority in VRAM. When VRAM utilization exceeds 85%, Arc's resource manager shall gracefully evict or offload local vision model tensors to host system RAM (CPU GGUF) or remote gateways, strictly preventing GPU driver resets (TDR) or session-terminating OOM kills.  
*Source: ADR-0007*

### REQ-AGENT-006: Scoped Browser Session Bridging & Ephemeral Workspaces
Autonomous web tasks shall execute in a dedicated Managed Arc Browser Profile with credentials scoped via the TPM2 vault, preventing filesystem lock conflicts with the user's primary browser profile and preventing cross-domain session cookie leakage. When encountering unpassable 2FA/CAPTCHA challenges, Arc shall execute an interactive authentication handover, spotlighting the window and waiting for user completion.  
*Source: ADR-0009*

### REQ-AGENT-007: CDP Automation Stealth & Fingerprint Masking
All automated browser instances shall strip automation flags (`--enable-automation`), sanitize runtime navigator properties (`navigator.webdriver = undefined`), and match native user TLS client hello fingerprints to prevent bot-detection barriers from obstructing valid user intents.  
*Source: ADR-0009*

### REQ-AGENT-008: Instant Hardware Input Preemption (< 1ms)
Physical user input from keyboard or pointer devices shall instantaneously override and suspend virtual autonomous input. If physical pointer movement exceeds 5 pixels or any physical key is pressed, the compositor shall suspend the ghost pointer in less than 1 millisecond without input jitter or race conditions.  
*Source: ADR-0004*

### REQ-AGENT-009: Autonomous Execution Budget & Idempotency Circuit Breaker
Autonomous task execution loops shall enforce a maximum budget of 15 atomic steps per plan without human checkpoint re-authorization. If an identical action fails twice consecutively, the system shall trip an idempotency circuit breaker, freeze execution, and request human guidance.  
*Source: Architecture §2.2, §5*

### REQ-AGENT-010: Offline Cognitive Degradation & Local-First Fallback
In the event of network partition or cloud provider unavailability, Arc shall gracefully downgrade Tier-1 planning to an on-disk local quantized model (3B–8B), maintaining operational capability for local system administration, scripting, and desktop navigation without blocking the user interface.  
*Source: Architecture §2.2*

---

## 6. Ambient Voice Duplex (`REQ-AUDIO`)

### REQ-AUDIO-001: Native PipeWire Audio Node
Arc shall connect directly as an active streaming node in the Linux PipeWire pro-audio graph, bypassing legacy audio daemon emulation layers.  
*Source: Architecture §7*

### REQ-AUDIO-002: Automatic Acoustic Media Ducking
When the sensory plane detects human speech onset via Voice Activity Detection, Arc shall attenuate all active audio output streams by 70% with smooth volume easing, restoring full volume when the speech transaction concludes.  
*Source: Architecture §7*

### REQ-AUDIO-003: Sub-200ms Conversational Latency & Transport Budgets
The complete voice roundtrip—from user speech termination, through streaming local transcription, reflex intent classification, and streaming neural text-to-speech synthesis—shall not exceed 200ms for conversational acknowledgments on local wired audio sinks (speakers, 3.5mm, USB DAC). For high-latency Bluetooth wireless transports (A2DP/HFP), the roundtrip SLA shall not exceed 400ms.  
*Source: Architecture §7, ADR-0008*

### REQ-AUDIO-004: Peer-Level Conversational Persona
The acoustic voice output shall maintain a calm, competent, collaborative persona, addressing the user personally and maintaining cross-modal continuity between spoken words and on-screen typography.  
*Source: Architecture §7*

### REQ-AUDIO-005: Hardware Acoustic Echo Cancellation & Barge-In
The audio sensory node shall integrate WebRTC Acoustic Echo Cancellation (AEC) using Arc's master speaker output as a reference signal, digitally canceling system speech and media from the microphone feed to enable natural voice interruption ("barge-in") with < 3ms input buffer latency.  
*Source: Architecture §7, ADR-0008*

### REQ-AUDIO-006: Dynamic AEC Sink Sensing & Headphone Bypass
When the active output sink is determined to be a headphone or in-ear monitor (via 3.5mm impedance sensing, USB DAC enumeration, or Bluetooth device profiles), the audio pipeline shall dynamically bypass the AEC DSP module, eliminating comb filtering, phase coloration, and 15ms of filter latency.  
*Source: ADR-0008*

---

## 7. Safety and Deterministic Enforcement (`REQ-SAF`)

### REQ-SAF-001: Strict Separation of Decision and Execution
No language model or probabilistic agent shall possess direct, unmediated operating system authority. All mutations must be formulated as structured proposals submitted to the Policy Broker.  
*Source: Architecture §2.3*

### REQ-SAF-002: Two-Dimensional Authorization
The Policy Broker shall evaluate every action proposal against a two-dimensional matrix of explicit capability (resource $\times$ operation) and clearance risk level. Missing or ambiguous capabilities shall fail closed.  
*Source: Architecture §2.3*

### REQ-SAF-003: Transaction Partitioning (Class-R vs. Class-I)
All proposed system mutations shall be deterministically partitioned:
1. **Class-R (Reversible Local Actions)**: File mutations, package installations, service restarts, and driver configurations shall execute in isolated staged sandboxes with automated pre-flight checkpoints, health check verification, and automatic rollback upon failure.
2. **Class-I (Irreversible External Actions)**: Web form submissions, emails, external network requests, and remote pushes shall strictly require an explicit Visual Cryptographic Approval Gate rendered on the spatial canvas detailing the exact destination, payload diff, and target origin prior to dispatch. Class-I actions shall never be automatically committed.  
*Source: Architecture §2.3, ADR-0006, ADR-0011*

### REQ-SAF-004: Hardware-Backed Credential Vault
Authentication tokens, SSH keys, and persistent browser session cookies shall be stored in an encrypted local vault backed by the platform TPM2, and shall never be passed into external model context windows.  
*Source: Architecture §2.3, ADR-0009*

### REQ-SAF-005: Data Provenance & Context Taint Tracking (`TaintedContext`)
Any text, document, or DOM element ingested from external sources (web pages, emails, untrusted files, network sockets) shall be assigned an immutable cryptographic `TaintedContext` tag. Any plan or tool invocation derived from a `TaintedContext` shall be barred from accessing the TPM2 Credential Vault, reading local private keys (`~/.ssh`, `~/.gnupg`), or initiating external network egress without a dedicated, human-confirmed visual approval gate.  
*Source: ADR-0011*

### REQ-SAF-006: Origin-Bound Cryptographic Credential Vault
The TPM2 Credential Vault shall enforce cryptographic origin isolation. Stored session cookies and authentication tokens shall only be released to engine pipes whose destination TLS connection and URL origin have been verified by the compositor to match the credential's target domain. Cross-origin credential requests shall be rejected and logged to the security audit trail.  
*Source: ADR-0009, ADR-0011*

### REQ-SAF-007: Guardian Emergency Diagnostic Override
To prevent fail-closed deadlocks during self-healing system recovery when telemetry is stale or broken, the system shall provide an Emergency Diagnostic Override. Triggered by a physical hardware chord (`Super + Escape` held for 3 seconds) or local root/biometric verification, the override enables an isolated diagnostic recovery transaction with Guardian invariants scoped strictly to emergency recovery boundaries.  
*Source: ADR-0011*
