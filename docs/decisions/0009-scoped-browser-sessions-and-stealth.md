# ADR-0009: Scoped Browser Session Bridging, Automation Stealth, and Interactive Authentication Handover

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Autonomous web navigation in Arc (such as checking email, monitoring job boards, or fetching research papers) confronts three severe runtime friction points:
1. **Browser Profile Mutex Locks**: Modern web browsers (Chromium, Firefox) lock their user data directory with filesystem locks (`SingletonLock` / SQLite process locks). Attempting to attach an automated runner directly to the user's primary profile causes process crashes or database corruption.
2. **Session Security & 2FA Walls**: Spawning isolated, temporary browser profiles forces the user to log in and resolve SMS/authenticator two-factor authentication (2FA) or CAPTCHAs repeatedly on every automated mission. Conversely, copying raw profile folders risks exfiltrating unneeded sensitive cookies (e.g., banking or personal credentials).
3. **Anti-Bot Detection**: Standard automation tools (Playwright, Selenium, raw CDP) trigger anti-bot defenses (Cloudflare Turnstile, Akamai, Datadome) by exposing `navigator.webdriver = true` and static automation command-line flags.

## Decision

**Arc will implement a dedicated Managed Browser Profile architecture with scoped TPM2 session cookie synchronization, automated CDP stealth sanitization, and interactive authentication handovers.**

1. **The Managed Arc Browser Profile**:
   - Arc operates an isolated, persistent browser profile directory (`~/.local/share/arc/browser/`) separate from the user's daily desktop browser.
   - Authorized session cookies (e.g., Indeed session tokens, Webmail credentials) are synchronized securely via Arc's local TPM2-backed Credential Vault. Only cookies explicitly granted to a specialist domain are mounted into the browser session.

2. **CDP Stealth & Fingerprint Sanitization**:
   - Automated browser processes strip `--enable-automation` and `--disable-blink-features`.
   - Pre-navigation initialization scripts override `navigator.webdriver` to `undefined`, randomize canvas/WebGL micro-noise, and match native TLS Client Hello fingerprints to emulate genuine user interaction.

3. **Interactive Authentication Handover**:
   - When the sensory plane detects an unpassable challenge (Cloudflare Turnstile, reCAPTCHA, SMS 2FA prompt, or hardware security key prompt):
     - Arc immediately halts the ghost pointer and transitions the task into `AwaitingUserAuth`.
     - The compositor racks focus to the browser window and gently illuminates its frame in soft amber light.
     - Arc speaks or displays a calm prompt: *"Shane, please complete the verification on Indeed."*
     - The sensory plane monitors cookie store updates and DOM URL transitions. The moment authentication is validated, Arc smoothly resumes autonomous navigation.

## Consequences

**Positive:**
- Eliminates profile lock crashes completely; user can browse freely while Arc automates tasks in parallel.
- Scoped cookie vault prevents credential leakage across unrelated web domains.
- Drastically minimizes CAPTCHA challenges via CDP stealth.
- Graceful 2FA handover provides seamless human-in-the-loop assistance without breaking workflow continuity.

**Negative:**
- Initial setup requires logging in once inside the Arc managed browser profile for each target service.
- Modern anti-bot heuristics evolve continuously and require periodic fingerprint updates.

## Related
- `architecture.md` §2.3, §6
- `requirements.md` REQ-AGENT-006, REQ-AGENT-007
- `decisions/0004-dual-channel-computer-use-visual-choreography.md`
- `decisions/0006-deterministic-enforcement-plane.md`
