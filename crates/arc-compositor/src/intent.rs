use std::time::{Duration, Instant};

/// Full canonical welcome prompt displayed on boot
pub const CANONICAL_PROMPT: &str = "Welcome to Arc, what would you like to do today?";

/// Tracks biological typing cadence and user zero-input intent buffer
#[derive(Debug)]
pub struct IntentManager {
    first_frame: Option<Instant>,
    pub user_input: String,
    pub is_committed: bool,
    pub status_message: Option<String>,
}

impl IntentManager {
    pub fn new() -> Self {
        Self {
            first_frame: None,
            user_input: String::new(),
            is_committed: false,
            status_message: None,
        }
    }

    /// Elapsed time since first render (the cinematic clock must start when
    /// the first frame hits the display, not at process spawn — wgpu init
    /// takes seconds and would eat the darkness hold).
    pub fn elapsed(&self) -> Duration {
        match self.first_frame {
            Some(t0) => t0.elapsed(),
            None => Duration::ZERO,
        }
    }

    /// Called by the renderer on its first frame to start the clock.
    pub fn start_clock(&mut self) {
        if self.first_frame.is_none() {
            self.first_frame = Some(Instant::now());
        }
    }

    /// Returns the slice of the canonical prompt visible at the current elapsed time.
    /// Simulates biological typing cadence with pauses at punctuation.
    pub fn visible_prompt(&self) -> &str {
        let elapsed_ms = self.elapsed().as_millis() as u64;

        // Cinematic pacing: 3s of pure darkness, ARC blooms in from 3–15s;
        // the prompt begins typing as the bloom crests (~14s), so text
        // arrives while light is still resolving.
        const INITIAL_STILLNESS_MS: u64 = 14000;
        if elapsed_ms < INITIAL_STILLNESS_MS {
            return "";
        }

        let typing_time = elapsed_ms - INITIAL_STILLNESS_MS;
        let mut cumulative_ms = 0u64;
        let mut visible_len = 0usize;

        for (i, c) in CANONICAL_PROMPT.char_indices() {
            let char_delay = match c {
                ',' => 240,
                '?' => 380,
                ' ' => 50,
                _ => 36,
            };
            cumulative_ms += char_delay;
            if cumulative_ms <= typing_time {
                visible_len = i + c.len_utf8();
            } else {
                break;
            }
        }

        &CANONICAL_PROMPT[..visible_len]
    }

    /// True if the welcome prompt has finished streaming in
    pub fn is_prompt_complete(&self) -> bool {
        self.visible_prompt().len() == CANONICAL_PROMPT.len()
    }

    /// Append typed character from physical keyboard
    pub fn push_char(&mut self, c: char) {
        if !self.is_committed {
            self.user_input.push(c);
            self.status_message = None;
        }
    }

    /// Remove last character (Backspace with kinetic echo)
    pub fn pop_char(&mut self) {
        if !self.is_committed {
            self.user_input.pop();
            self.status_message = None;
        }
    }

    /// Commit the current intent buffer on Enter
    pub fn commit(&mut self) {
        if self.user_input.trim().is_empty() {
            return;
        }
        self.is_committed = true;
        let query = self.user_input.trim();
        self.status_message = Some(format!(
            "Arc Reflex > Intent acknowledged: \"{}\". Synthesizing spatial stage...",
            query
        ));
    }

    /// Clear the intent buffer (Escape)
    pub fn clear(&mut self) {
        self.user_input.clear();
        self.is_committed = false;
        self.status_message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_lifecycle() {
        let mut intent = IntentManager::new();
        assert_eq!(intent.user_input, "");
        assert!(!intent.is_committed);
        assert_eq!(intent.status_message, None);

        // Simulate typing "what is system memory?"
        for c in "what is system memory?".chars() {
            intent.push_char(c);
        }
        assert_eq!(intent.user_input, "what is system memory?");

        // Backspace test
        intent.pop_char();
        assert_eq!(intent.user_input, "what is system memory");

        // Commit intent
        intent.commit();
        assert!(intent.is_committed);
        assert!(intent.status_message.is_some());
        let msg = intent.status_message.as_ref().unwrap();
        assert!(msg.contains("what is system memory"));

        // Clear resets
        intent.clear();
        assert_eq!(intent.user_input, "");
        assert!(!intent.is_committed);
        assert_eq!(intent.status_message, None);
    }

    #[test]
    fn test_empty_commit_noop() {
        let mut intent = IntentManager::new();
        intent.commit();
        assert!(!intent.is_committed);
        assert_eq!(intent.status_message, None);
    }
}

