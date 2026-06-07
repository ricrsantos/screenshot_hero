use std::path::{Path, PathBuf};

/// Builds the auto-export destination path for a source image.
///
/// The result is `{stem}{suffix}.png` in the same directory as `source`.
/// Auto-export always produces PNG regardless of the source format.
pub fn build_auto_export_path(source: &Path, suffix: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let new_name = format!("{stem}{suffix}.png");
    source.with_file_name(new_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_auto_export_path_png() {
        let source = PathBuf::from("/home/u/Screenshots/shot.png");
        let result = build_auto_export_path(&source, "_shero");
        assert_eq!(result, PathBuf::from("/home/u/Screenshots/shot_shero.png"));
    }

    #[test]
    fn test_build_auto_export_path_jpeg_source() {
        let source = PathBuf::from("/home/u/Screenshots/shot.jpg");
        let result = build_auto_export_path(&source, "_shero");
        assert_eq!(result, PathBuf::from("/home/u/Screenshots/shot_shero.png"));
    }

    #[test]
    fn test_build_auto_export_path_no_extension() {
        let source = PathBuf::from("/home/u/Screenshots/shot");
        let result = build_auto_export_path(&source, "_shero");
        assert_eq!(result, PathBuf::from("/home/u/Screenshots/shot_shero.png"));
    }
}
