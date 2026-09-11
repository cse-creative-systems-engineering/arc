# ADR-0006: Retaining the Deterministic Enforcement Plane from Aios

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

While Aios v0.1 struggled with the "bolted-on application" paradigm in its desktop layer, it solved one of the hardest problems in AI systems engineering: **how to grant AI real system agency without giving it root and hoping for the best.**

Specifically, Aios authored and validated:
1. **The Policy Broker**: A two-dimensional authorization engine evaluating explicit Capabilities against clearance Risk Levels (Risk 1 to 4).
2. **The Infrastructure Guardian**: A deterministic veto system enforcing immutable safety invariants that cannot be overridden by conversational persuasion.
3. **The Staged Transaction Executor**: Checkpoint $\rightarrow$ Stage $\rightarrow$ Health Check $\rightarrow$ Commit or Rollback, preventing broken machine states.
4. **Hardware and Domain Specialists**: Typed tools wrapping device discovery and driver operations.

Discarding this verified enforcement core when building Arc would introduce unacceptable safety regressions.

## Decision

**Arc will adopt and adapt the deterministic Enforcement Plane from Aios directly into its core execution engine.**

1. All autonomous actions proposed by Arc's cognitive planner (whether web actions, package installs, or driver configuration) must be formulated as structured action proposals submitted to the Policy Broker.
2. The Infrastructure Guardian will retain veto power over high-risk actions.
3. System mutations will be executed via staged transactions with verified rollback checkpoints.
4. The Enforcement Plane will run as an internal supervisor module compiled directly into the Arc display and runtime daemon.

## Consequences

**Positive:**
- Preserves the 449-test verification heritage and security guarantees pioneered in Aios.
- Ensures that Arc's expansive autonomous agency (web navigation, ML pipeline scaffolding, system management) remains safe, auditable, and reversible.
- Provides clear visual consent boundaries: when an action exceeds Risk Level 2, the Arc compositor renders a native cryptographic approval gate on the screen.

**Negative:**
- Actions cannot be executed immediately; they must pass through the staging, pre-flight verification, and policy broker validation lifecycle.

## Related
- `architecture.md` §2.3
- `requirements.md` REQ-SAF-001, REQ-SAF-002, REQ-SAF-003
- `glossary.md` (Policy Broker, Staged Transaction Executor)
