# ADR-0004: Dual-Channel Computer Use and Visual Choreography

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Recent AI systems have attempted "Computer Use" by treating the desktop as an image: taking screenshots, running multimodal vision models to predict $(x, y)$ screen coordinates, and injecting mouse/keyboard events.

In practice, this approach suffers from fatal flaws:
1. **Extreme Fragility**: Sub-pixel layout shifts, minor font rendering differences, or transient popups cause vision models to hallucinate coordinates and misclick.
2. **High Latency**: Roundtrip screenshot encoding, vision inference, and coordinate mapping take 1–3 seconds per action.
3. **Loss of Trust**: When an agent clicks blindly, the user cannot anticipate what it is about to do, leading to anxiety during destructive or financial actions.

Conversely, headless API scraping (running background cURL/Playwright scripts) provides speed and reliability, but is completely invisible to the user, eliminating situational awareness.

## Decision

**Arc decouples autonomous action into two synchronized channels: the Deterministic Control Channel and the Visual Choreography Channel.**

1. **Control Channel (100% Reliable)**:
   - For web applications (e.g., Indeed, Webmail), Arc controls browser windows directly via the Chrome DevTools Protocol (CDP) or native browser extension IPC.
   - For native Linux desktop applications, Arc queries the semantic widget hierarchy via the AT-SPI accessibility bus.
   - Target elements, inputs, and buttons are resolved deterministically using semantic selectors, DOM nodes, and accessibility IDs—not visual coordinate guessing.

2. **Visual Choreography Channel (Human Transparency)**:
   - When an action occurs, the compositor choreographs the visual experience on the screen.
   - Using Wayland's virtual pointer protocol (`zwp_virtual_pointer_v1`), Arc animates a luminous ghost cursor across the screen with natural cubic-bezier physics to the target element.
   - The targeted element illuminates, the click activates with visible spring feedback, and text fills with natural typing rhythm.
   - The user observes the entire workflow at human-comprehensible speed.

3. **Privileged Protocol Gating (Security Invariant)**:
   - Protocols that simulate input (`zwp_virtual_pointer_v1` and `zwp_virtual_keyboard_v1`) are strictly classified as privileged.
   - The compositor verifies peer credentials (`SO_PEERCRED` on Linux Unix domain sockets) on client connection. Only internal Arc supervisor threads or verified system worker processes are permitted to bind to virtual input interfaces.
   - Any unauthenticated third-party Wayland client attempting to bind to virtual pointer or virtual keyboard protocols is immediately terminated with a protocol error (`WL_DISPLAY_ERROR_INVALID_OBJECT`).

4. **Physical Input Preemption Invariant (Instant Hardware Veto)**:
   - To eliminate input race conditions between human and AI, physical hardware events take absolute, instantaneous priority over virtual input.
   - The moment `libinput` reports physical pointer movement exceeding a 5-pixel threshold or any physical key actuation:
     - The compositor executes an **Instant Ghost Pause** in $< 1\text{ms}$.
     - The virtual pointer stream is suspended, the ghost cursor dissolves into a soft pulsing halo, and all physical seat focus yields unconditionally to the human.
     - The autonomous task pauses execution cleanly without dropping state until the human resumes or cancels the action.

## Consequences

**Positive:**
- Eliminates misclicks completely: actions are grounded in deterministic DOM/AT-SPI tree state.
- Preserves full human transparency: the user watches Arc navigate and fill forms without guessing what the machine is doing.
- Eliminates pointer jitter and fight-for-control races through deterministic physical input preemption (< 1ms).
- Closes the arbitrary input-injection vulnerability: third-party client apps cannot hijack the user's mouse or keyboard through Wayland.

**Negative:**
- Requires maintaining CDP and AT-SPI connection drivers for target applications.
- Visual choreography intentionally introduces slight pacing delays (e.g., 200–400ms per step) so the human eye can follow along, though this can be accelerated or made headless for trusted repetitive tasks.

## Related
- `architecture.md` §6, §6.1
- `requirements.md` REQ-AGENT-001, REQ-AGENT-002, REQ-AGENT-008
- `glossary.md` (Dual-Channel Control, Ghost Pointer)
