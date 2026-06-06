use serde::{Deserialize, Serialize};

use crate::annotations::Annotation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheroProject {
    pub version: u32,
    pub source_image: SourceImageRecord,
    pub annotations: Vec<Annotation>,
    pub view_state: ViewState,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceImageRecord {
    pub path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewState {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub created_at: String,
    pub modified_at: String,
    pub app_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::{AnnotationKind, AnnotationStyle, Rect};
    use uuid::Uuid;

    fn sample_project() -> SheroProject {
        SheroProject {
            version: 1,
            source_image: SourceImageRecord {
                path: "/tmp/screenshot.png".to_string(),
                width: 1920,
                height: 1080,
            },
            annotations: vec![Annotation {
                id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
                kind: AnnotationKind::Rectangle,
                bounds: Rect {
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                },
                style: AnnotationStyle::default(),
            }],
            view_state: ViewState {
                zoom: 1.5,
                pan_x: -32.0,
                pan_y: 48.0,
            },
            metadata: ProjectMetadata {
                created_at: "2026-06-06T12:00:00Z".to_string(),
                modified_at: "2026-06-06T13:00:00Z".to_string(),
                app_version: "0.1.0".to_string(),
            },
        }
    }

    #[test]
    fn shero_project_round_trips_through_json() {
        let original = sample_project();
        let json = serde_json::to_string(&original).expect("serialize SheroProject");
        let restored: SheroProject =
            serde_json::from_str(&json).expect("deserialize SheroProject");
        assert_eq!(original, restored);
    }
}
