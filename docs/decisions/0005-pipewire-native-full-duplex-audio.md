# ADR-0005: PipeWire-Native Full-Duplex Audio

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Conversational voice in traditional desktop environments has been crippled by high latency, clumsy push-to-talk buttons, and lack of integration with system audio:
1. Legacy audio servers (PulseAudio, ALSA) introduce buffer latencies and struggle with dynamic stream routing.
2. If background music or media is playing, voice assistants either fail to hear the user over the audio or deafeningly blare their responses over the user's music.
3. Push-to-talk buttons destroy conversational flow, while cloud-based speech-to-text systems introduce 1–2 second latency spikes that kill conversational intimacy.

Modern Linux features **PipeWire**, a professional-grade real-time audio and video processing graph with low latency and granular stream routing.

## Decision

**Arc integrates directly into the Linux PipeWire graph as an active audio processing node with continuous Voice Activity Detection (VAD) and automatic media ducking.**

The audio architecture consists of:
1. **PipeWire Client Node**: Arc establishes a native stream node in the PipeWire graph, receiving low-latency microphone frames and injecting synthesized speech directly into the output sink.
2. **Silero VAD**: A lightweight Voice Activity Detection model processes microphone frames continuously with near-zero CPU impact, detecting speech onset within 10ms.
3. **Automatic Media Ducking**: When speech onset is detected, Arc sends a volume attenuation signal through the PipeWire mixer, smoothly dipping all active media playback streams by 70%. When speech concludes, volume returns smoothly to 100%.
4. **Streaming Local Neural Pipeline**:
   - Speech-to-Text: Local streaming transcription (e.g., Whisper / Moonshine) pipes partial tokens into the Tier-0 reflex engine.
   - Text-to-Speech: High-fidelity local streaming synthesis (e.g., Kokoro-82M) begins streaming audio buffers to PipeWire within 30ms of first token generation.

## Consequences

**Positive:**
- Enables effortless, hands-free conversation with sub-200ms perceptual latency.
- Automatic media ducking ensures the user can converse comfortably even while listening to music or watching media.
- Runs completely offline without cloud audio telemetry or privacy leakage.

**Negative:**
- Requires PipeWire as a mandatory system dependency (standard on modern Linux distributions, but excludes legacy PulseAudio-only environments).
- Pinned neural speech models (Whisper/Kokoro) require ~300–500MB of dedicated system RAM.

## Related
- `architecture.md` §7
- `requirements.md` REQ-AUDIO-001, REQ-AUDIO-002, REQ-AUDIO-003
- `glossary.md` (PipeWire Node, Audio Ducking)
