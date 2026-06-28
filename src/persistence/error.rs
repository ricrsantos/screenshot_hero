use std::fmt;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
    MissingSourceImage(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::Io(err) => write!(f, "I/O error: {err}"),
            PersistenceError::Json(err) => write!(f, "JSON error: {err}"),
            PersistenceError::UnsupportedVersion(version) => {
                write!(f, "unsupported project version: {version}")
            }
            PersistenceError::MissingSourceImage(path) => {
                write!(f, "source image not found: {path}")
            }
        }
    }
}
