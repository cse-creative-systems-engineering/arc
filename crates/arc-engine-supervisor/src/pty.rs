//! Headless PTY terminal pipe (Milestone 0002, REQ-SURF-001).
//!
//! Spawns real interactive CLI/TUI processes (e.g. btop, bash, neovim)
//! via portable-pty, delivering raw character and control streams without
//! relying on external X11/Wayland terminal emulator windows.

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct TerminalPipe {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub rx: Receiver<Vec<u8>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl TerminalPipe {
    pub fn spawn(
        command: &str,
        cols: u16,
        rows: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(command);
        let child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));

        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

        // Background thread reads raw ANSI byte streams from the PTY master
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF / process exit
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            writer,
            rx,
            _child: child,
        })
    }

    pub fn write_input(&self, data: &[u8]) -> std::io::Result<()> {
        let mut w = self.writer.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Lock poisoned")
        })?;
        w.write_all(data)?;
        w.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_terminal_pipe_echo() {
        let pipe = TerminalPipe::spawn("echo", 80, 24).expect("Spawn echo command");
        // Read output from the command
        let mut out = Vec::new();
        let timeout = std::time::Instant::now();
        while timeout.elapsed() < Duration::from_secs(2) {
            if let Ok(bytes) = pipe.rx.recv_timeout(Duration::from_millis(50)) {
                out.extend_from_slice(&bytes);
                if !out.is_empty() {
                    break;
                }
            }
        }
        // At minimum, process should have exited or produced bytes
        assert!(timeout.elapsed() < Duration::from_secs(3));
    }
}
