//! Tier-0/1 Reflex bridge (architecture.md §2.2).
//!
//! Committing an intent on the zero-input canvas dispatches it to the
//! configured OpenRouter model and streams tokens back for the kinetic
//! status narrative. Development default: `thinkingmachines/inkling:free`.
//! Key/config loads from `~/.config/arc/arc.env` — never committed (REQ-SAF-004).

use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

#[derive(Debug)]
pub struct ReflexConfig {
    pub api_key: String,
    pub model: String,
}

impl ReflexConfig {
    /// Load from ~/.config/arc/arc.env (KEY=VALUE lines).
    pub fn load() -> Option<Self> {
        let path = dirs_home().join(".config/arc/arc.env");
        let text = std::fs::read_to_string(path).ok()?;
        let mut api_key = None;
        let mut model = None;
        for line in text.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("OPENROUTER_API_KEY=") {
                api_key = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("ARC_MODEL=") {
                model = Some(v.trim().to_string());
            }
        }
        Some(Self {
            api_key: api_key?,
            model: model.unwrap_or_else(|| "thinkingmachines/inkling:free".to_string()),
        })
    }
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

/// Handle to an in-flight reflex query. Tokens arrive on a channel the
/// render loop polls — never blocking the 120Hz path (ADR-0012 §2).
pub struct ReflexEngine {
    config: ReflexConfig,
    rx: Option<Receiver<ReflexEvent>>,
    pub response_so_far: String,
}

#[derive(Debug)]
pub enum ReflexEvent {
    Token(String),
    Done,
    Failed(String),
}

impl ReflexEngine {
    pub fn new(config: ReflexConfig) -> Self {
        Self {
            config,
            rx: None,
            response_so_far: String::new(),
        }
    }

    /// Dispatch a committed intent on a background thread.
    pub fn submit(&mut self, intent: String) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.response_so_far.clear();

        let config = ReflexConfig {
            api_key: self.config.api_key.clone(),
            model: self.config.model.clone(),
        };

        std::thread::spawn(move || {
            let outcome = query_blocking(&config, &intent);
            match outcome {
                Ok(response) => {
                    // Visual channel: stream one character at a time so the
                    // kinetic narrative types letter-by-letter (REQ-UX-002).
                    // The full sentence arrives instantly for the audio
                    // channel; this pacing only shapes the visual stream.
                    for ch in response.chars() {
                        if tx.send(ReflexEvent::Token(ch.to_string())).is_err() {
                            return;
                        }
                        // Biological cadence: longer pause after sentence
                        // ends, brief breath after commas.
                        let delay = match ch {
                            '.' | '!' | '?' => 260,
                            ',' | ';' | ':' => 120,
                            _ => 26,
                        };
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                    }
                    let _ = tx.send(ReflexEvent::Done);
                }
                Err(e) => {
                    let _ = tx.send(ReflexEvent::Failed(e));
                }
            }
        });
    }

    /// Poll for new tokens; appends to `response_so_far`. Returns true if
    /// the stream is still active.
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else { return false };
        let mut active = true;
        loop {
            match rx.try_recv() {
                Ok(ReflexEvent::Token(t)) => self.response_so_far.push_str(&t),
                Ok(ReflexEvent::Done) => {
                    active = false;
                    break;
                }
                Ok(ReflexEvent::Failed(e)) => {
                    tracing::warn!("reflex query failed: {e}");
                    self.response_so_far = format!("Reflex error: {e}");
                    active = false;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    active = false;
                    break;
                }
            }
        }
        if !active {
            self.rx = None;
        }
        active
    }
}

fn query_blocking(config: &ReflexConfig, intent: &str) -> Result<String, String> {
    // One-shot tokio runtime on this worker thread — the compositor render
    // loop stays winit/wgpu-native and never awaits (ADR-0012 zero-alloc path).
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(async {
        let body = ChatRequest {
            model: config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "You are Arc, an intent-first operating substrate. \
                              Respond like a trusted engineering colleague. \
                              Your response is consumed two ways simultaneously: \
                              read aloud by a voice synthesizer and typed letter-by-letter \
                              on a cinematic canvas. Therefore: \
                              (1) Structure every response as short, declarative sentences \
                              optimized for speech — no markdown, no bullet glyphs, no code \
                              fences unless code itself was requested. \
                              (2) Lead with the direct answer in the first sentence. \
                              (3) Keep total length under 80 words unless the task demands more."
                        .into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: intent.to_string(),
                },
            ],
            stream: false,
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("HTTP-Referer", "https://github.com/cse-creative-systems-engineering/arc")
            .header("X-Title", "Arc Compositor")
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        let parsed: ChatResponse = resp.json().await.map_err(|e| format!("{status}: {e}"))?;

        if let Some(err) = parsed.error {
            return Err(err.message);
        }
        parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message.map(|m| m.content))
            .ok_or_else(|| format!("{status}: empty response"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse_missing_file() {
        // With HOME pointed at an empty dir, load must return None (no panic).
        let old = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/tmp/arc-test-empty-home");
        std::fs::create_dir_all("/tmp/arc-test-empty-home").unwrap();
        assert!(ReflexConfig::load().is_none());
        if let Some(old) = old {
            std::env::set_var("HOME", old);
        }
    }

    #[test]
    fn test_config_parse_valid_file() {
        let dir = "/tmp/arc-test-valid-home/.config/arc";
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(format!("{dir}/arc.env"), "OPENROUTER_API_KEY=sk-test\nARC_MODEL=test/model\n").unwrap();
        let old = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/tmp/arc-test-valid-home");
        let cfg = ReflexConfig::load().expect("config must load");
        assert_eq!(cfg.api_key, "sk-test");
        assert_eq!(cfg.model, "test/model");
        if let Some(old) = old {
            std::env::set_var("HOME", old);
        }
    }

    #[test]
    fn test_poll_inactive_fresh() {
        let mut engine = ReflexEngine::new(ReflexConfig {
            api_key: "x".into(),
            model: "m".into(),
        });
        assert!(!engine.poll());
        assert!(engine.response_so_far.is_empty());
    }
}
