//! Scene Graph Intermediate Representation (`ArcSceneIR`) (Milestone 0002, ADR-0012 §1, REQ-SCENE-004).
//!
//! A strictly typed, bounded, and schema-validated AST for dynamic domain scenes
//! (e.g. Living Bookmark Wall, Codebase Landscape, Acoustic Listening Room).
//! Constrained primitives guarantee immunity against layout loops, unbounded
//! recursion, and GPU rasterizer DOS.

use serde::{Deserialize, Serialize};

pub const MAX_SCENE_DEPTH: usize = 8;
pub const MAX_CONTAINER_CHILDREN: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneNode {
    SurfaceContainer {
        id: String,
        depth: f32,
        children: Vec<SceneNode>,
    },
    GridLayout {
        columns: u32,
        spacing: f32,
        children: Vec<SceneNode>,
    },
    ThumbnailGrid {
        items: Vec<BookmarkThumbnail>,
    },
    KineticStream {
        id: String,
        label: String,
    },
    VectorCard {
        id: String,
        title: String,
        width: f32,
        height: f32,
        opacity: f32,
        interactive: bool,
    },
    DataStreamNode {
        stream_id: String,
        metric_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookmarkThumbnail {
    pub id: String,
    pub title: String,
    pub url: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SceneValidationError {
    #[error("Tree depth exceeded maximum allowed limit of {0}")]
    TreeDepthExceeded(usize),
    #[error("Container exceeded maximum children limit of {0}")]
    TooManyChildren(usize),
    #[error("Transparent interaction blocker detected (opacity={0}, interactive=true)")]
    TransparentBlocker(f32),
}

impl SceneNode {
    /// Validates the AST against security and performance invariants (REQ-SCENE-004, ADR-0012 §1).
    pub fn validate(&self) -> Result<(), SceneValidationError> {
        self.validate_recursive(1)
    }

    fn validate_recursive(&self, current_depth: usize) -> Result<(), SceneValidationError> {
        if current_depth > MAX_SCENE_DEPTH {
            return Err(SceneValidationError::TreeDepthExceeded(MAX_SCENE_DEPTH));
        }

        match self {
            SceneNode::SurfaceContainer { children, .. } | SceneNode::GridLayout { children, .. } => {
                if children.len() > MAX_CONTAINER_CHILDREN {
                    return Err(SceneValidationError::TooManyChildren(MAX_CONTAINER_CHILDREN));
                }
                for child in children {
                    child.validate_recursive(current_depth + 1)?;
                }
            }
            SceneNode::ThumbnailGrid { items } => {
                if items.len() > MAX_CONTAINER_CHILDREN {
                    return Err(SceneValidationError::TooManyChildren(MAX_CONTAINER_CHILDREN));
                }
            }
            SceneNode::VectorCard {
                opacity,
                interactive,
                ..
            } => {
                if *interactive && *opacity <= 0.001 {
                    return Err(SceneValidationError::TransparentBlocker(*opacity));
                }
            }
            SceneNode::KineticStream { .. } | SceneNode::DataStreamNode { .. } => {}
        }

        Ok(())
    }
}

/// Fallback AST generation when synthesis or validation fails (Fail-Closed).
pub fn fail_closed_scene(error_message: &str) -> SceneNode {
    SceneNode::KineticStream {
        id: "fallback-diagnostic".into(),
        label: format!("Arc Scene Fault: {}", error_message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bookmark_wall_ir() {
        let wall = SceneNode::SurfaceContainer {
            id: "bookmark-wall".into(),
            depth: 0.0,
            children: vec![
                SceneNode::ThumbnailGrid {
                    items: vec![
                        BookmarkThumbnail {
                            id: "bm-1".into(),
                            title: "Hacker News".into(),
                            url: "https://news.ycombinator.com".into(),
                            width: 320.0,
                            height: 200.0,
                        },
                        BookmarkThumbnail {
                            id: "bm-2".into(),
                            title: "Docs".into(),
                            url: "https://docs.rs".into(),
                            width: 320.0,
                            height: 200.0,
                        },
                    ],
                },
            ],
        };

        assert!(wall.validate().is_ok());
    }

    #[test]
    fn test_reject_transparent_blocker() {
        let malicious_card = SceneNode::VectorCard {
            id: "clickjack".into(),
            title: "Invisible Overlay".into(),
            width: 1920.0,
            height: 1080.0,
            opacity: 0.0,
            interactive: true,
        };

        assert_eq!(
            malicious_card.validate(),
            Err(SceneValidationError::TransparentBlocker(0.0))
        );
    }

    #[test]
    fn test_reject_excessive_depth() {
        let mut root = SceneNode::VectorCard {
            id: "leaf".into(),
            title: "Leaf".into(),
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            interactive: false,
        };

        for i in 0..10 {
            root = SceneNode::SurfaceContainer {
                id: format!("depth-{}", i),
                depth: i as f32,
                children: vec![root],
            };
        }

        assert_eq!(
            root.validate(),
            Err(SceneValidationError::TreeDepthExceeded(MAX_SCENE_DEPTH))
        );
    }
}
