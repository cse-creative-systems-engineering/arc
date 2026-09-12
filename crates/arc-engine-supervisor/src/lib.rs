//! Arc Engine Supervisor (Milestone 0002, ADR-0010, REQ-SURF-001).
//!
//! Manages headless Engine Pipes (Chromium web runners, ANSI PTY terminals)
//! that execute background workloads and stream raw offscreen frame buffers
//! directly to the Arc compositor without traditional window chrome or title bars.

pub mod pty;
pub mod web;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngineId(pub u64);

#[derive(Debug, Clone)]
pub enum EngineKind {
    Terminal { command: String },
    Web { url: String },
}

#[derive(Debug, Clone)]
pub struct EngineDescriptor {
    pub id: EngineId,
    pub kind: EngineKind,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

pub struct EngineSupervisor {
    engines: Arc<RwLock<HashMap<EngineId, EngineDescriptor>>>,
    next_id: std::sync::atomic::AtomicU64,
}

impl EngineSupervisor {
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    pub fn allocate_id(&self) -> EngineId {
        EngineId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }

    pub async fn register(&self, descriptor: EngineDescriptor) {
        let mut map = self.engines.write().await;
        map.insert(descriptor.id, descriptor);
    }

    pub async fn unregister(&self, id: EngineId) -> Option<EngineDescriptor> {
        let mut map = self.engines.write().await;
        map.remove(&id)
    }

    pub async fn list(&self) -> Vec<EngineDescriptor> {
        let map = self.engines.read().await;
        map.values().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        let map = self.engines.read().await;
        map.len()
    }
}

impl Default for EngineSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_registration() {
        let supervisor = EngineSupervisor::new();
        let id1 = supervisor.allocate_id();
        let desc = EngineDescriptor {
            id: id1,
            kind: EngineKind::Terminal {
                command: "bash".into(),
            },
            title: "Bash Terminal Pipe".into(),
            width: 800,
            height: 600,
        };

        supervisor.register(desc.clone()).await;
        assert_eq!(supervisor.count().await, 1);

        let list = supervisor.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Bash Terminal Pipe");

        let removed = supervisor.unregister(id1).await;
        assert!(removed.is_some());
        assert_eq!(supervisor.count().await, 0);
    }
}
