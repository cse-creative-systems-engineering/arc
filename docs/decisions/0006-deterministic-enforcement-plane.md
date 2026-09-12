# ADR-0006: Deterministic Enforcement Plane (Policy Broker, Guardian, Staged Executor)

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Operating systems that integrate artificial intelligence face a central, existential paradox: **probabilistic models are capable of high-level reasoning and synthesis, but are fundamentally untrustworthy as security or safety boundaries.** 

A language model can be tricked by prompt injections, hallucinate non-existent system paths, or generate commands that destabilize the machine. Giving a reasoning agent root or unmediated shell access and hoping for the best is architectural negligence.

Conversely, sandboxing an AI so severely that it can only answer text questions defeats the entire vision of an intent-first autonomous operating substrate.

To move the machine reliably, Arc requires a deterministic enforcement plane: **no component gets to both decide what to do and possess the unrestricted authority to execute it.**

## Decision

**Arc establishes an immutable, deterministic Enforcement Plane as a first-class supervisor compiled directly into the runtime daemon.**

1. **Two-Dimensional Policy Broker ($Capability \times Clearance$)**:
   - The cognitive planner never executes operating system calls directly. It emits structured action proposals.
   - The Policy Broker validates that the specialist role possesses the explicit capability (resource $\times$ operation) and the required clearance level (Risk Level 1 to 4). Ambiguous or missing capabilities fail closed.

2. **The Infrastructure Guardian (Deterministic Safety Invariants)**:
   - A deterministic guardian module holds absolute veto power over proposed system transitions.
   - Invariants (boot chain integrity, kernel safety, credential boundaries, network firewall rules) cannot be bypassed by model reasoning or conversational persuasion.

3. **Transaction Partitioning (Class-R vs. Class-I)**:
   - System mutations are partitioned into:
     - **Class-R (Reversible Local Actions)**: Checkpoint $\rightarrow$ Stage in sandbox $\rightarrow$ Health Check $\rightarrow$ Commit or Rollback.
     - **Class-I (Irreversible External Actions)**: Web form submissions, emails, and external network mutations strictly require a native visual cryptographic consent gate rendered on the spatial canvas prior to dispatch.

4. **Internal Supervisor Architecture**:
   - The Enforcement Plane runs in-process as an isolated supervisor thread within the Arc daemon, communicating with engine pipes and the compositor via typed internal channels.

## Consequences

**Positive:**
- Complete safety guarantee: Arc will lose intelligence before it loses the ability to safely recover.
- Models may recommend and plan, but hard limits are enforced by code that cannot be talked into anything.
- Enables safe, expansive autonomous execution across local subsystems, developers' tools, and web services.
- Provides clear visual consent boundaries: high-risk and irreversible mutations require explicit human signoff on the canvas.

**Negative:**
- Actions cannot execute instantaneously; every mutating operation must pass capability evaluation, staging, and health check verification.

## Related
- `architecture.md` §2.3
- `requirements.md` REQ-SAF-001, REQ-SAF-002, REQ-SAF-003, REQ-SAF-005, REQ-SAF-006, REQ-SAF-007
- `decisions/0011-data-provenance-taint-tracking-and-irreversible-actions.md`
- `glossary.md` (Policy Broker, Staged Transaction Executor)
