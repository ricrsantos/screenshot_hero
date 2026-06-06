use std::fs;
use std::path::Path;

use crate::persistence::error::PersistenceError;
use crate::persistence::model::SheroProject;

pub fn load_project(path: &Path) -> Result<SheroProject, PersistenceError> {
    let contents = fs::read_to_string(path).map_err(PersistenceError::Io)?;
    let project: SheroProject =
        serde_json::from_str(&contents).map_err(PersistenceError::Json)?;
    if project.version != 1 {
        return Err(PersistenceError::UnsupportedVersion(project.version));
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::{AnnotationKind, AnnotationStyle, Rect};
    use crate::persistence::{ProjectMetadata, SourceImageRecord, ViewState};
    use uuid::Uuid;

    fn sample_project() -> SheroProject {
        SheroProject {
            version: 1,
            source_image: SourceImageRecord {
                path: "/tmp/screenshot.png".to_string(),
                width: 1920,
                height: 1080,
            },
            annotations: vec![crate::annotations::Annotation {
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

    fn write_temp_shero(contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("test-{}.shero", Uuid::new_v4()));
        std::fs::write(&path, contents).expect("write temp .shero file");
        path
    }

    #[test]
    fn load_project_deserializes_valid_shero_json() {
        let expected = sample_project();
        let json = serde_json::to_string(&expected).expect("serialize sample project");
        let path = write_temp_shero(&json);

        let loaded = load_project(&path).expect("load valid project");

        assert_eq!(loaded, expected);
    }

    #[test]
    fn load_project_rejects_unsupported_version() {
        let mut project = sample_project();
        project.version = 2;
        let json = serde_json::to_string(&project).expect("serialize project");
        let path = write_temp_shero(&json);

        let err = load_project(&path).expect_err("unsupported version should fail");

        assert!(matches!(
            err,
            PersistenceError::UnsupportedVersion(2)
        ));
    }

    #[test]
    fn load_project_rejects_malformed_json() {
        let path = write_temp_shero("{ not valid json");

        let err = load_project(&path).expect_err("malformed JSON should fail");

        assert!(matches!(err, PersistenceError::Json(_)));
    }
}
