use std::path::{Path, PathBuf};

use crate::models::{ImageData, SourceImage};

#[derive(Debug, PartialEq, Eq)]
pub enum LoadError {
    FileNotFound(PathBuf),
    UnsupportedFormat(PathBuf),
    DecodeFailed(String),
    InvalidUri(String),
}

pub struct FileLoader;

impl FileLoader {
    pub fn load_from_path(path: &Path) -> Result<ImageData, LoadError> {
        if !is_supported_extension(path) {
            return Err(LoadError::UnsupportedFormat(path.to_path_buf()));
        }

        if !path.exists() {
            return Err(LoadError::FileNotFound(path.to_path_buf()));
        }

        match gdk_pixbuf::Pixbuf::from_file(path) {
            Ok(pixbuf) => {
                let width = pixbuf.width();
                let height = pixbuf.height();
                let source = SourceImage {
                    path: path.to_path_buf(),
                    width,
                    height,
                };
                Ok(ImageData::from_pixbuf(pixbuf, source))
            }
            Err(error) => Err(LoadError::DecodeFailed(error.to_string())),
        }
    }

    pub fn load_from_uri(uri: &str) -> Result<ImageData, LoadError> {
        let path = glib::filename_from_uri(uri)
            .map(|(path, _)| path)
            .map_err(|error| LoadError::InvalidUri(error.to_string()))?;

        Self::load_from_path(&path)
    }
}

fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn test_load_png_valid_file() {
        let path = fixture_path("sample.png");
        let image = FileLoader::load_from_path(&path).expect("PNG fixture should load");

        assert!(image.width() > 0);
        assert!(image.height() > 0);
        assert_eq!(image.source().path, path);
    }

    #[test]
    fn test_load_jpeg_valid_file() {
        let path = fixture_path("sample.jpg");
        let image = FileLoader::load_from_path(&path).expect("JPEG fixture should load");

        assert!(image.width() > 0);
        assert!(image.height() > 0);
        assert_eq!(image.source().path, path);
    }

    #[test]
    fn test_load_unsupported_format() {
        let path = fixture_path("sample.bmp");
        let Err(error) = FileLoader::load_from_path(&path) else {
            panic!("BMP should be rejected");
        };

        assert_eq!(error, LoadError::UnsupportedFormat(path));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = fixture_path("does-not-exist.png");
        let Err(error) = FileLoader::load_from_path(&path) else {
            panic!("missing file should fail");
        };

        assert_eq!(error, LoadError::FileNotFound(path));
    }

    #[test]
    fn test_load_from_uri_valid() {
        let path = fixture_path("sample.png");
        let uri = glib::filename_to_uri(&path, None)
            .expect("fixture path should convert to URI")
            .to_string();
        let image = FileLoader::load_from_uri(&uri).expect("file URI should load");

        assert!(image.width() > 0);
        assert!(image.height() > 0);
        assert_eq!(image.source().path, path);
    }

    #[test]
    fn test_load_from_uri_invalid() {
        let Err(error) = FileLoader::load_from_uri("not-a-valid-uri") else {
            panic!("invalid URI should fail");
        };

        assert!(matches!(error, LoadError::InvalidUri(_)));
    }
}
