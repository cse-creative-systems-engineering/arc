# ADR-0003: Zero-Input Ambient Canvas

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Conventional desktop environments require explicit user interaction to acquire focus before input can be received:
1. The user must locate an application icon, dock item, or menu entry.
2. The user must click a search bar or open a dedicated terminal window.
3. The user must wait for the text cursor to blink inside an explicit input bounding box before typing.

This ceremony creates artificial friction between human thought and system execution. In DOS 6.22 or early command environments, the prompt was immediately present; in modern GUI systems, the prompt is buried under application windows.

## Decision

**Arc establishes the Zero-Input Ambient Canvas paradigm: the display surface itself is the global input buffer.**

### 1. Focus Arbitration & The Ambient Summon Chord
- **Default State (No Modal Window)**: All raw keyboard events from `libinput` route directly to the compositor's root node and into the Global Intent Buffer. No text boxes or focus clicks exist.
- **Client Window Focus**: When a client application window is active (e.g., Neovim, terminal, browser input), keystrokes route exclusively to that client's Wayland surface.
- **Ambient Summon Chord**: Pressing `Super` (Windows key) or double-tapping `Escape` immediately yields keyboard focus from the active client back to Arc's Global Intent Buffer. The background client windows smoothly dim by 30%, indicating the ambient canvas is listening.
- **Shortcut Preemption**: Client requests to inhibit compositor global shortcuts (`zwp_keyboard_shortcuts_inhibit_v1`) are denied by default unless explicitly granted by the Policy Broker.

### 2. Kinetic Typographic Pipeline
- Characters appear dead-center on the canvas rendered in oversized kinetic typography.
- Backspace applies physics-based spring transitions to remove glyphs.
- Pressing `Enter` commits the text to the Tier-0 reflex engine.

### 3. Internationalization and IME Architecture
- The Global Intent Buffer implements the server-side `zwp_text_input_v3` protocol internally.
- Input Method Editors (IMEs such as `fcitx5` or `ibus`) connect directly to the compositor's virtual input surface, rendering CJK pre-edit strings and candidate popup menus directly above the kinetic text stream without requiring an external toolkit widget.

## Consequences

**Positive:**
- Eliminates UI clutter: the desktop remains a pristine, calm OLED void until action is required.
- Zero-latency interaction: the user can walk up to the machine, begin typing immediately, and see results.
- Unifies command line, search bar, and chat prompt into an ambient physical canvas.
- Full global support for complex international scripts (CJK, Arabic, Hebrew RTL) through native Wayland IME integration.
- Instant, deterministic focus reclamation via the Ambient Summon Chord without requiring mouse interaction.

**Negative:**
- Implementing server-side `zwp_text_input_v3` adds state management complexity to the compositor input engine.
- Users accustomed to traditional desktop focus models must learn the `Super` or double-tap `Escape` summon convention.

**Neutral:**
- Seamlessly mirrors spoken audio input: whether the user speaks or types, the intent lands in the exact same processing pipeline.

## Related
- `architecture.md` §3.1, §3.2
- `requirements.md` REQ-UX-001, REQ-UX-002, REQ-UX-005
- `glossary.md` (Ambient Canvas, Zero-Input, Global Intent Buffer)
