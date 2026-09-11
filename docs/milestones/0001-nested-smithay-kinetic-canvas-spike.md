# Milestone 0001: Nested Smithay Kinetic Canvas Spike

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `requirements.md`, ADR-0001, ADR-0002, ADR-0003  

## Objective

Build the initial proof-of-concept for the Arc display substrate. This milestone validates the boot sequence, procedural breathing shader, kinetic typography engine, and zero-input global keyboard capture running inside a nested window on the developer's existing desktop (requiring zero reboots).

---

## Scope and Deliverables

1. **Rust Workspace Scaffold**:
   - Initialize the `arc` Rust workspace with the `arc-compositor` crate.
   - Configure dependencies: `smithay`, `wgpu`, `cosmic-text`, `glyphon` / `vello`, `tokio`, `calloop`.

2. **Nested Smithay Backend**:
   - Initialize a Smithay compositor using `smithay::backend::winit`.
   - Open a borderless or managed test window (1920x1080) running at native display refresh rates.

3. **Obsidian Void & Procedural Shader**:
   - Render a pure OLED black canvas (`#000000`).
   - Implement a procedural WGPU fragment shader:
     - The word **ARC** fades in slowly via a smooth sine-wave easing function, scaling to dominant screen proportions.
     - Decays smoothly to a persistent 5% alpha ambient watermark that pulses faintly.

4. **Kinetic Typography Engine**:
   - Render the canonical welcome prompt:  
     > *Welcome to Arc, what would you like to do today?*
   - Text streams in character-by-character with calibrated biological typewriter cadence (varying pause lengths between punctuation and spaces).
   - Displayed in oversized literary display typography spanning the center third of the viewport.

5. **Zero-Input Keystroke Echo**:
   - Intercept keyboard events directly from the compositor root window without a text-input box or click-to-focus requirement.
   - Typed characters appear immediately on the canvas below or replacing the prompt.
   - Backspace triggers smooth glyph removal with physical spring dynamics.

---

## Acceptance Criteria

### Vignette 1: The Boot and Ambient Fade
- **Action**: Run `cargo run -p arc-compositor`.
- **Expected Result**: A window opens to pure pitch black. Within 1.5 seconds, the word "ARC" smoothly blooms into view across the screen in luminous titanium/warm white light, then gently fades back to an ethereal 5% watermark. The welcome prompt streams in letter-by-letter dead center.

### Vignette 2: The Zero-Input Test
- **Action**: Without clicking any text box or pressing any activation key, the developer types: `"what is the system memory usage?"`.
- **Expected Result**: Keystrokes appear immediately in real-time vector typography on the screen. Pressing Backspace smoothly erases characters. Pressing Enter locks the intent buffer.

---

## Verification Plan

```bash
# 1. Check workspace compilation
cargo check --workspace

# 2. Run unit tests on typography and buffer state
cargo test --workspace

# 3. Launch the nested compositor window
cargo run --bin arc-compositor
```
