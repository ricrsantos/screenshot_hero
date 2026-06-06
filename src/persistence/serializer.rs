use std::fs;
use std::path::Path;

use super::error::PersistenceError;
use super::model::SheroProject;

/// Flatpak / xdg-desktop-portal grants write access only to the exact file
/// returned by the save dialog. Creating a sibling `.tmp` and renaming fails
/// (no "neighbor" permission), leaving orphaned `.xdp-*.tmp-*` staging files.
pub fn is_portal_document_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s.contains(".xdp-") {
        return true;
    }
    s.starts_with("/run/user/") && s.contains("/doc/")
}

pub fn save_project(path: &Path, project: &SheroProject) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(project).map_err(PersistenceError::Json)?;

    if is_portal_document_path(path) {
        fs::write(path, json).map_err(PersistenceError::Io)?;
    } else {
        let mut temp_path = path.as_os_str().to_os_string();
        temp_path.push(".tmp");

        fs::write(&temp_path, json).map_err(PersistenceError::Io)?;
        fs::rename(&temp_path, path).map_err(PersistenceError::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::{Annotation, AnnotationKind, AnnotationStyle, Rect};
    use crate::persistence::{ProjectMetadata, SourceImageRecord, ViewState};
    use std::fs;
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
    fn portal_document_paths_are_detected() {
        assert!(is_portal_document_path(Path::new(
            "/run/user/1000/doc/.xdp-cap2.shero.tmp-ifkkL1"
        )));
        assert!(is_portal_document_path(Path::new(
            "/run/user/1000/doc/abc123/myproject.shero"
        )));
        assert!(!is_portal_document_path(Path::new("/tmp/myproject.shero")));
        assert!(!is_portal_document_path(Path::new(
            "/home/user/Pictures/myproject.shero"
        )));
    }

    #[test]
    fn save_project_writes_valid_shero_file() {
        let path = std::env::temp_dir().join(format!("shero_test_{}.shero", Uuid::new_v4()));
        let project = sample_project();

        save_project(&path, &project).expect("save project");

        let contents = fs::read_to_string(&path).expect("read saved file");
        let restored: SheroProject =
            serde_json::from_str(&contents).expect("deserialize saved file");
        assert_eq!(project, restored);

        let _ = fs::remove_file(&path);
    }
}
