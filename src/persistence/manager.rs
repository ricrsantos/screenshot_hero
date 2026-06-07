use std::path::{Path, PathBuf};

use chrono::Utc;

use super::deserializer;
use super::error::PersistenceError;
use super::model::SheroProject;
use super::serializer;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct ProjectManager {
    pub current_path: Option<PathBuf>,
    pub auto_save_enabled: bool,
    pub created_at: Option<String>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            current_path: None,
            auto_save_enabled: true,
            created_at: None,
        }
    }

    pub fn save(&mut self, path: &Path, mut project: SheroProject) -> Result<(), PersistenceError> {
        self.apply_metadata(&mut project);
        self.current_path = Some(path.to_path_buf());
        serializer::save_project(path, &project)
    }

    pub fn save_as(&mut self, path: &Path, project: SheroProject) -> Result<(), PersistenceError> {
        self.save(path, project)
    }

    pub fn open(&mut self, path: &Path) -> Result<SheroProject, PersistenceError> {
        let project = deserializer::load_project(path)?;
        self.current_path = Some(path.to_path_buf());
        self.created_at = Some(project.metadata.created_at.clone());
        Ok(project)
    }

    pub fn maybe_auto_save(&self, mut project: SheroProject) {
        if !self.auto_save_enabled {
            return;
        }
        let Some(path) = self.current_path.as_ref() else {
            return;
        };

        let now = Utc::now().to_rfc3339();
        if let Some(created_at) = &self.created_at {
            project.metadata.created_at = created_at.clone();
        }
        project.metadata.modified_at = now;
        project.metadata.app_version = APP_VERSION.to_string();

        if let Err(err) = serializer::save_project(path, &project) {
            log::warn!("auto-save failed: {err}");
        }
    }

    fn apply_metadata(&mut self, project: &mut SheroProject) {
        let now = Utc::now().to_rfc3339();

        if self.created_at.is_none() {
            let created_at = if project.metadata.created_at.is_empty() {
                now.clone()
            } else {
                project.metadata.created_at.clone()
            };
            self.created_at = Some(created_at.clone());
            project.metadata.created_at = created_at;
        } else {
            project.metadata.created_at = self.created_at.clone().unwrap();
        }

        project.metadata.modified_at = now;
        project.metadata.app_version = APP_VERSION.to_string();
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::{Annotation, AnnotationKind, AnnotationStyle, Rect};
    use crate::persistence::{ProjectMetadata, SourceImageRecord, ViewState};
    use std::fs;
    use std::thread;
    use std::time::Duration;
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
                created_at: String::new(),
                modified_at: String::new(),
                app_version: String::new(),
            },
        }
    }

    fn temp_shero_path() -> PathBuf {
        std::env::temp_dir().join(format!("shero_mgr_test_{}.shero", Uuid::new_v4()))
    }

    #[test]
    fn save_preserves_created_at_and_advances_modified_at() {
        let path = temp_shero_path();
        let mut manager = ProjectManager::new();
        let project = sample_project();

        manager.save(&path, project).expect("first save");

        let first = deserializer::load_project(&path).expect("read after first save");
        assert!(!first.metadata.created_at.is_empty());
        assert!(!first.metadata.modified_at.is_empty());
        assert_eq!(
            manager.created_at.as_deref(),
            Some(first.metadata.created_at.as_str())
        );

        thread::sleep(Duration::from_secs(1));

        let project2 = sample_project();
        manager.save(&path, project2).expect("second save");

        let second = deserializer::load_project(&path).expect("read after second save");
        assert_eq!(second.metadata.created_at, first.metadata.created_at);
        assert_ne!(second.metadata.modified_at, first.metadata.modified_at);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn maybe_auto_save_without_path_writes_no_file() {
        let watch_dir = std::env::temp_dir().join(format!("shero_autosave_{}", Uuid::new_v4()));
        fs::create_dir_all(&watch_dir).expect("create watch dir");

        let manager = ProjectManager::new();
        assert!(manager.current_path.is_none());

        manager.maybe_auto_save(sample_project());

        let entries: Vec<_> = fs::read_dir(&watch_dir)
            .expect("read watch dir")
            .filter_map(Result::ok)
            .collect();
        assert!(entries.is_empty());

        let _ = fs::remove_dir(&watch_dir);
    }
}
