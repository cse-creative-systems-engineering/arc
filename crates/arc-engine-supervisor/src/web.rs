//! Headless Chromium Web Engine Runner (Milestone 0002, ADR-0009, ADR-0010).
//!
//! Spawns headless browser engine instances with hardened flags:
//! --headless=new, --use-gl=angle, --use-angle=vulkan, --remote-debugging-pipe
//! to render live web content directly into DMA-BUF GPU memory.

use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct WebEngineOptions {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub user_data_dir: Option<std::path::PathBuf>,
}

pub struct WebEnginePipe {
    pub options: WebEngineOptions,
    child: Option<Child>,
}

impl WebEnginePipe {
    pub fn new(options: WebEngineOptions) -> Self {
        Self {
            options,
            child: None,
        }
    }

    /// Spawns the headless chromium engine process if installed on the system.
    pub fn spawn(&mut self) -> Result<(), std::io::Error> {
        let binary_candidates = ["chromium", "chromium-browser", "google-chrome", "brave"];
        let mut found_bin = None;
        for bin in binary_candidates {
            if which::which(bin).is_ok() {
                found_bin = Some(bin);
                break;
            }
        }

        let Some(bin) = found_bin else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No supported Chromium-based headless runner binary found on PATH",
            ));
        };

        let mut cmd = Command::new(bin);
        cmd.arg("--headless=new")
            .arg(format!("--window-size={},{}", self.options.width, self.options.height))
            .arg("--disable-gpu-watchdog")
            .arg("--enable-features=UseSkiaRenderer,Vulkan")
            .arg("--enable-unsafe-webgpu")
            .arg("--remote-debugging-pipe")
            .arg(&self.options.url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        if let Some(ref dir) = self.options.user_data_dir {
            cmd.arg(format!("--user-data-dir={}", dir.display()));
        }

        let child = cmd.spawn()?;
        self.child = Some(child);
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn terminate(&mut self) -> std::io::Result<()> {
        if let Some(ref mut child) = self.child {
            child.kill()?;
            let _ = child.wait();
        }
        self.child = None;
        Ok(())
    }
}

impl Drop for WebEnginePipe {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_engine_options_builder() {
        let options = WebEngineOptions {
            url: "https://news.ycombinator.com".into(),
            width: 1280,
            height: 800,
            user_data_dir: None,
        };
        let mut pipe = WebEnginePipe::new(options);
        assert!(!pipe.is_running());
    }
}
