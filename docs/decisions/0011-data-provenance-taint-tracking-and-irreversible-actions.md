# ADR-0011: Data Provenance, Context Taint Tracking, and Irreversible Action Gating

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Extending autonomous agency to external web services and real-time document analysis introduces four existential security and execution hazards:

1. **Indirect Prompt Injection (Confused Deputy)**: When Arc browses websites or ingests emails, third-party content may embed adversarial prompt injection strings (e.g., hidden text instructing the agent to exfiltrate `~/.ssh` keys or transfer tokens). Without strict data provenance, the agent's reasoning plane can be hijacked into proposing harmful system actions.
2. **The Rollback Illusion for External Actions**: While local filesystem mutations can be checkpointed and rolled back cleanly, external network actions (submitting job applications, sending emails, posting messages, purchasing items) are physically and legally immutable. A transaction executor cannot "roll back" an HTTP POST request.
3. **Flat Credential Vault Vulnerabilities**: If all stored session cookies and API keys reside in a single flat vault, a compromised specialist thread or prompt-injected agent navigating one domain (e.g., Indeed) could request credentials for an unrelated high-value domain (e.g., AWS or GitHub).
4. **Guardian Telemetry Livelock**: In a pure fail-closed security architecture, if a driver or network interface breaks and telemetry becomes stale, the Infrastructure Guardian will veto self-healing repair plans, creating an unrecoverable deadlock.

## Decision

**Arc establishes strict Data Provenance with Context Taint Tracking, partitions all transactions into Reversible (Class-R) and Irreversible (Class-I) actions, enforces Origin-Bound Credential Isolation, and provides an Emergency Diagnostic Override.**

### 1. Data Provenance & Context Taint Tracking
- Any text, document, or DOM element ingested from external sources (web pages, emails, untrusted files, network sockets) is assigned an immutable cryptographic **`TaintedContext`** tag.
- If a proposed plan or tool call incorporates data from a `TaintedContext`:
  - The Policy Broker automatically revokes all autonomous execution clearance.
  - The plan is barred from accessing the TPM2 Credential Vault, reading local private keys (`~/.ssh`, `~/.gnupg`, `~/.config`), or initiating external network egress without a dedicated, explicit visual confirmation gate displayed directly on the screen.

### 2. Transaction Classification (Class-R vs. Class-I)
The Staged Transaction Executor formally splits all actions into two deterministic classes:
- **Class-R (Reversible Local Mutations)**: Local file edits, package installations, service restarts, and driver configurations. Executed via automated pre-flight checkpoints, health verification, and automatic rollback upon failure.
- **Class-I (Irreversible External Mutations)**: Web form submissions, emails, remote git pushes, and financial transactions. **These actions can never be automatically committed.** They require an unambiguous Visual Cryptographic Approval Gate rendered on the spatial canvas detailing the exact destination, payload diff, and target origin before dispatch.

### 3. Origin-Bound Credential Isolation
- The local TPM2 Credential Vault enforces cryptographic origin matching.
- A credential (e.g., an Indeed session cookie) will *only* be injected into an engine pipe whose active TLS connection and URL origin have been verified by the compositor to match `https://*.indeed.com`.
- Cross-origin credential requests are rejected and logged to the security audit trail.

### 4. Guardian Emergency Diagnostic Override
- To prevent fail-closed deadlocks when repairing broken subsystems whose telemetry is stale:
- Arc provides a physical **Emergency Recovery Path**. Holding `Super + Escape` for 3 seconds or providing local biometric/root authentication enables an isolated Diagnostic Recovery Mode, allowing targeted driver or network reload transactions with Guardian invariants scoped strictly to emergency boundaries.

## Consequences

**Positive:**
- Eliminates the risk of automated data exfiltration via indirect prompt injection through web or email content.
- Prevents accidental, irreversible real-world actions by strictly gating Class-I network mutations behind visual consent gates.
- Origin-bound vaulting prevents cross-domain credential harvesting.
- Emergency override breaks deadlocks when repairing failed system hardware with stale telemetry.

**Negative:**
- Users must explicitly approve Class-I external mutations, preventing 100% hands-off automation for actions like final email dispatch or checkout.
- Taint tracking adds metadata overhead to the context management pipeline.

## Related
- `architecture.md` §2.3, §5, §6
- `requirements.md` REQ-SAF-002, REQ-SAF-003, REQ-SAF-005, REQ-SAF-006, REQ-SAF-007
- `decisions/0006-deterministic-enforcement-plane.md`
- `decisions/0009-scoped-browser-sessions-and-stealth.md`
