# ADR-0008: Native Microphone Sensory Pipeline and Acoustic Echo Cancellation

**Status:** Accepted  
**Date:** 2026-09-11  

## Context

Conversational voice interaction on laptops and workstations faces severe physical and architectural hurdles:
1. **Chassis Proximity and Acoustic Feedback (Echo Loop)**: On modern laptops, speakers and built-in microphone arrays share the same physical chassis. When the system speaks or plays audio, the microphone picks up the speaker output. Without real-time echo cancellation, the agent hears its own voice, hallucinates echoes, or interrupts itself.
2. **Buffer Latency**: Traditional desktop audio APIs route through multiple daemon abstraction layers, adding 40–100ms of buffer latency before audio samples reach userland.
3. **Continuous Inference Power Drain**: Running a full neural speech recognition model continuously on open microphone buffers quickly exhausts CPU/GPU resources and drains laptop battery.

To achieve conversational fluidity where the user can speak naturally or interrupt the machine at any moment, Arc requires direct hardware microphone access with real-time acoustic isolation.

## Decision

**Arc will establish a native PipeWire sensory audio node with in-graph WebRTC Acoustic Echo Cancellation (AEC) and low-power Voice Activity Detection (VAD).**

1. **Direct ALSA Hardware Ring Buffers via PipeWire**:
   - Arc connects as a real-time client node in PipeWire, requesting low-latency 128-sample buffer cycles (1.3ms to 2.6ms buffer latency at 16kHz/48kHz).
   - Audio is streamed via direct shared-memory ring buffers from physical hardware codecs (Intel SOF, AMD ACP, Realtek HDA, USB microphones).

2. **Hardware/In-Graph Acoustic Echo Cancellation (AEC)**:
   - Arc instantiates the PipeWire WebRTC AEC filter module (`libpipewire-module-echo-cancel`).
   - The master speaker output stream is routed back into the AEC node as the acoustic reference channel.
   - The system digitally cancels Arc's own synthesized voice and background media playback from the microphone signal in real time, delivering a pristine voice isolation stream.
   - This enables **natural barge-in**: the user can interrupt Arc mid-sentence ("Wait, stop!") and Arc immediately detects the human voice without acoustic distortion.

3. **Two-Stage Audio Awakening (Silero VAD $\rightarrow$ Streaming STT)**:
   - Microphone PCM chunks are continuously evaluated by **Silero VAD** in 512-sample chunks (< 1ms CPU time, < 0.1% CPU utilization).
   - Only when speech onset is validated does the pipeline awaken the streaming speech-to-text model (Whisper / Moonshine).

## Consequences

**Positive:**
- Acoustic roundtrip latency from physical sound waves to PCM memory drops to < 3 milliseconds.
- Pristine acoustic echo cancellation allows reliable voice interruption even during loud media playback.
- Ultra-low idle CPU utilization (< 0.1%) during ambient listening.
- Works identically across internal laptop microphone arrays and external USB/XLR interfaces.

**Negative:**
- Requires PipeWire's echo-cancellation module, which introduces a minor DSP processing latency (~10–15ms) on the filtered stream.
- Severe acoustic clipping (e.g., maximum volume speaker distortion on budget hardware) can challenge software echo cancellation algorithms.

## Related
- `architecture.md` §2.1, §7
- `requirements.md` REQ-AUDIO-001, REQ-AUDIO-002, REQ-AUDIO-005
- `decisions/0005-pipewire-native-full-duplex-audio.md`
