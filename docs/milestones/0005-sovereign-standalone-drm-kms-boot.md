# Milestone 0005: Sovereign Standalone DRM/KMS Boot

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `requirements.md`, ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, Milestones 0001–0004  

## Objective

Transition Arc from a nested development window into a sovereign standalone operating substrate. This milestone integrates direct hardware seat management via `libseat` and `udev`, Direct Rendering Manager (DRM) and Kernel Mode Setting (KMS) page flipping, multi-output display enumeration, native virtual terminal (VT) switching, and systemd service integration, completely replacing traditional desktop environments (GNOME, KDE, X11).

---

## Scope and Deliverables

1. **DRM/KMS Hardware Backend**:
   - Implement the `smithay::backend::drm` driver in `arc-compositor`.
   - Acquire primary GPU DRM master status, allocating hardware scanout planes with direct zero-copy page flipping.
   - Support hardware variable refresh rates (Adaptive Sync / FreeSync / G-Sync) up to native monitor capabilities (120Hz/144Hz/240Hz).

2. **Native Seat Management (`libseat` & `udev`)**:
   - Manage non-root hardware seat access using `seatd` or `systemd-logind`.
   - Implement udev device discovery and dynamic hotplug handling for displays, keyboards, mice, and graphic tablets.
   - Support seamless VT switching (`Ctrl+Alt+F1`–`F7`), cleanly dropping and restoring DRM master status without visual artifacts.

3. **Multi-Monitor Frustum & Unified 3D Canvas**:
   - Enumerate multiple physical displays and map them into a continuous 3D spatial coordinate space.
   - Allow spatial surfaces and camera focal points to traverse multiple physical displays with zero tearing or stutter.

4. **Standalone Boot Integration (`arc.service`)**:
   - Author systemd unit files (`/etc/systemd/system/arc.service`) configured for target runlevels.
   - Implement seamless boot splash handoff from the Linux kernel / Plymouth directly into Arc's procedural obsidian void bloom.

5. **Substrate Soak Testing & Resource Stability**:
   - 48-hour continuous soak test under simulated multi-engine workloads (concurrent 4K video playback, headless browser crawls, and terminal compilation).
   - Zero memory leaks, stable VRAM footprint, and strict adherence to the Tier-0 compositor memory guarantee.

---

## Acceptance Criteria

### Vignette 1: Cold Boot to Obsidian Void
- **Action**: Power on the machine with `arc.service` enabled as the primary display manager.
- **Expected Result**: System boots silently past firmware. The screen smoothly transitions from kernel pitch black into the blooming "ARC" titanium watermark in under 1.5 seconds. The central prompt streams in letter-by-letter. No desktop docks, login managers, or window bars appear.

### Vignette 2: Hotplug and Multi-Monitor Spatial Extension
- **Action**: Connect a secondary high-refresh-rate monitor via DisplayPort while Arc is running.
- **Expected Result**: Udev detects the display within 50ms. Arc automatically configures KMS scanout and extends the 3D kinetic canvas seamlessly across both panels.

---

## Verification Plan

```bash
# 1. Run DRM/KMS backend unit tests
cargo test -p arc-compositor --features drm

# 2. Test VT switching and seat release in isolated VT
sudo arc-compositor --backend drm --seat seat0

# 3. Verify systemd service definition and dependencies
systemd-analyze verify arc.service
```
