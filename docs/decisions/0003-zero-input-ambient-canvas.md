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

When the system boots or when no client application window holds modal keyboard focus:
1. All raw keyboard events from `libinput` are captured directly by the compositor's root node.
2. The UI contains no permanent search boxes, command bars, or input rectangles.
3. As the user types, characters appear dead-center on the canvas rendered in oversized kinetic typography.
4. Backspace applies physics-based spring transitions to remove glyphs.
5. Pressing `Enter` commits the text to the Tier-0 reflex engine.

## Consequences

**Positive:**
- Eliminates UI clutter: the desktop remains a pristine, calm OLED void until action is required.
- Zero-latency interaction: the user can walk up to the machine, begin typing immediately, and see results.
- Unifies command line, search bar, and chat prompt into an ambient physical canvas.

**Negative:**
- Requires careful focus arbitration: when a client window (e.g., a text editor or browser) is active, keystrokes must route exclusively to that client unless an ambient escape chord (such as `Super` or double-tap `Escape`) is pressed.
- Novel interaction model requires clear initial typographic affordances to instruct first-time users that typing is active.

**Neutral:**
- Seamlessly mirrors spoken audio input: whether the user speaks or types, the intent lands in the exact same processing pipeline.

## Related
- `architecture.md` §3.1, §3.2
- `requirements.md` REQ-UX-001, REQ-UX-002
- `glossary.md` (Ambient Canvas, Zero-Input, Global Intent Buffer)
