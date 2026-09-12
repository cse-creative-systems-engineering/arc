# Milestone 0003: Dual-Channel Agency & Deterministic Enforcement

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `requirements.md`, ADR-0004, ADR-0006, ADR-0009, ADR-0011, Milestone 0002  

## Objective

Establish Arc's dual-channel agency and deterministic safety core. This milestone decouples visible user interface choreography (luminous ghost pointer, tactile highlights, narrative pulse) from deterministic execution channels (CDP, AT-SPI), enforces socket-level authentication on Wayland virtual input protocols, integrates the Policy Broker and Infrastructure Guardian, tracks context taint (`TaintedContext`), and enforces Class-R rollback and Class-I cryptographic approval gates.

---

## Scope and Deliverables

1. **Deterministic Policy Broker & Guardian (`arc-enforcement`)**:
   - Implement the `PolicyBroker` state machine evaluating proposed actions against explicit capability matrices and clearance levels.
   - Implement the `InfrastructureGuardian` to monitor system-level invariants (disk quotas, network egress limits, VRAM thresholds, critical system paths).
   - Implement Class-R transaction isolation with automated pre-flight snapshotting and rollback.

2. **Privileged Wayland Input Gating**:
   - Gate server-side `zwp_virtual_pointer_v1` and `zwp_virtual_keyboard_v1` implementations.
   - Inspect client socket credentials via Linux `SO_PEERCRED`. Only authenticated internal Arc agent threads are granted protocol bindings; unauthorized connections terminate immediately.
   - Implement instant physical preemption: any physical mouse movement (> 5px delta) or physical keypress suspends autonomous virtual input in $< 1\text{ms}$.

3. **Dual-Channel Ghost Pointer & Visual Choreography**:
   - Render a luminous vector ghost pointer gliding with cubic-bezier spring easing over target UI elements.
   - Implement tactile feedback: target DOM/UI elements illuminate gently as the ghost cursor arrives.
   - Render the narrative pulse: a single line of streaming monospace text beneath the active surface narrating real-time agent intent.

4. **Data Provenance & Context Taint Tracking (`TaintedContext`)**:
   - Tag all external data ingested from web sessions, emails, or untrusted files with an immutable `TaintedContext` wrapper.
   - Cryptographically bar tainted contexts from accessing the TPM2 credential vault or dispatching external network requests without human authorization.

5. **Class-I Visual Approval Gate & 2FA Interactive Handover**:
   - Synthesize a prominent visual cryptographic confirmation dialog for Class-I irreversible actions (external emails, transactions, remote code pushes) detailing target origin, exact payload diff, and cryptographic digest.
   - Detect 2FA/CAPTCHA challenges in headless browser engine pipes, smoothly bringing the surface front-and-center and handing input over to the user.

---

## Acceptance Criteria

### Vignette 1: Privileged Socket Gating & Instant Preemption
- **Action**: An external third-party process attempts to bind to Arc's `zwp_virtual_pointer_v1` Wayland global.
- **Expected Result**: The compositor checks `SO_PEERCRED`, detects an unauthenticated UID/PID, rejects the bind, and terminates the client socket with an audit log.
- **Action 2**: While Arc's internal ghost cursor is moving across a surface, the user nudges the physical mouse.
- **Expected Result**: Autonomous movement freezes within $< 1\text{ms}$ and physical input claims immediate focus without latency or pointer conflict.

### Vignette 2: Class-I Cryptographic Signoff Gate
- **Action**: An autonomous agent completes a multi-step web task and prepares to submit an online order or dispatch an email.
- **Expected Result**: Execution pauses. A high-contrast visual approval card appears over the surface displaying the recipient, payload, and SHA-256 hash. The action does not proceed until the user physically confirms via `Enter` or click.

---

## Verification Plan

```bash
# 1. Run policy broker matrix and guardian tests
cargo test -p arc-enforcement

# 2. Test virtual input peercred authentication
cargo test -p arc-compositor --test virtual_input_gating

# 3. Test taint propagation and Class-I gate invariants
cargo test -p arc-enforcement --test taint_tracking
```
