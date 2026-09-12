# Milestone 0004: PipeWire Native Voice Duplex & Conversational Reflex

**Status:** Draft Specification  
**Depends on:** `architecture.md`, `requirements.md`, ADR-0005, ADR-0008, Milestone 0001  

## Objective

Establish Arc's ambient voice duplex. This milestone integrates Arc directly into Linux's PipeWire pro-audio graph as a native audio node, implements hardware-synchronized Acoustic Echo Cancellation (AEC) and Voice Activity Detection (VAD), streams speech through local STT/TTS models, and delivers conversational barge-in with automatic media ducking and multi-modal intent fusion onto the kinetic canvas.

---

## Scope and Deliverables

1. **PipeWire Native Audio Node (`arc-audio`)**:
   - Create a native client node within the PipeWire graph via `libpipewire-rs`.
   - Ingest microphone capture streams with hardware timestamping.
   - Output synthesized agent voice and system auditory feedback directly to the graph.

2. **WebRTC Acoustic Echo Cancellation (AEC3)**:
   - Loop back audio output signals as reference streams into WebRTC AEC3.
   - Subtract speaker output from microphone input in real time, preventing agent self-transcription and feedback loops.
   - Implement dynamic headphone bypass when playback sinks report headphone/IEM connections.

3. **Low-Latency VAD & Conversational Barge-In**:
   - Process audio chunks through Silero VAD within 10ms.
   - When human speech onset is detected during agent speech or running media, trigger immediate conversational barge-in:
     - Instantly abort running agent TTS playback.
     - Smoothly duck running media audio by 70% with a 20ms exponential curve.

4. **Streaming STT & Local Neural TTS**:
   - Stream pre-filtered audio frames into a low-latency local speech-to-text pipeline (Moonshine / Whisper.cpp).
   - Stream generated reply tokens into local neural TTS (Kokoro-82M / Piper), emitting the first audio chunk in $< 30\text{ms}$.
   - Maintain total conversational round-trip latency below 200ms.

5. **Multi-Modal Intent Fusion**:
   - Live transcription stream echoes directly into the canvas Global Intent Buffer in real time.
   - Voice and physical keyboard inputs merge seamlessly into the unified reflex pipeline.

---

## Acceptance Criteria

### Vignette 1: Conversational Barge-In During Media Playback
- **Action**: Play audio through a media engine pipe while Arc is speaking a response. The user says: *"Arc, hold on"*.
- **Expected Result**: Silero VAD detects human speech within 10ms. Running media ducks immediately, Arc halts speech synthesis mid-word, and the system transitions to active listening without picking up speaker echo.

### Vignette 2: Speech-to-Canvas Vector Echo
- **Action**: The user speaks: *"Show me my local git status"*.
- **Expected Result**: Glyphs stream across the obsidian canvas in real time as the words are spoken, seamlessly populating the central intent buffer.

---

## Verification Plan

```bash
# 1. Run audio node and AEC loopback unit tests
cargo test -p arc-audio

# 2. Benchmark VAD detection latency
cargo test -p arc-audio --bench vad_latency

# 3. Test PipeWire graph integration with mock audio sink
cargo run --bin arc-audio-test
```
