use std::fmt;
use std::fs;
use std::path::Path;

use gdk_pixbuf::Pixbuf;

use crate::persistence::is_portal_document_path;

#[derive(Debug)]
pub enum ExportError {
    SaveFailed(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::SaveFailed(msg) => write!(f, "save failed: {msg}"),
        }
    }
}

pub fn export_png(pixbuf: &Pixbuf, path: &Path) -> Result<(), ExportError> {
    save_pixbuf(pixbuf, path, "png", &[])
}

pub fn export_jpeg(pixbuf: &Pixbuf, path: &Path) -> Result<(), ExportError> {
    let rgb = to_rgb_pixbuf(pixbuf)?;
    save_pixbuf(&rgb, path, "jpeg", &[("quality", "90")])
}

/// Flatpak grants write access only to the exact portal staging path from the save
/// dialog. `Pixbuf::savev` writes a temp sibling and renames, which fails silently
/// and leaves a 0-byte staging file — encode to memory and write directly instead.
fn save_pixbuf(
    pixbuf: &Pixbuf,
    path: &Path,
    type_: &str,
    options: &[(&str, &str)],
) -> Result<(), ExportError> {
    if is_portal_document_path(path) {
        let bytes = pixbuf
            .save_to_bufferv(type_, options)
            .map_err(|err| ExportError::SaveFailed(err.to_string()))?;
        fs::write(path, bytes).map_err(|err| ExportError::SaveFailed(err.to_string()))
    } else {
        pixbuf
            .savev(path, type_, options)
            .map_err(|err| ExportError::SaveFailed(err.to_string()))
    }
}

/// JPEG encoders (including glycin in Flatpak) require RGB without alpha.
fn to_rgb_pixbuf(pixbuf: &Pixbuf) -> Result<Pixbuf, ExportError> {
    if !pixbuf.has_alpha() {
        return pixbuf
            .copy()
            .ok_or_else(|| ExportError::SaveFailed("failed to copy pixbuf".into()));
    }

    let width = pixbuf.width();
    let height = pixbuf.height();
    let rowstride = width * 3;
    let mut data = vec![0u8; (height * rowstride) as usize];

    let src = pixbuf.read_pixel_bytes();
    let src_data = src.as_ref();
    let src_rowstride = pixbuf.rowstride() as usize;
    let src_channels = pixbuf.n_channels() as usize;

    for y in 0..height as usize {
        for x in 0..width as usize {
            let src_off = y * src_rowstride + x * src_channels;
            let dst_off = y * rowstride as usize + x * 3;
            data[dst_off] = src_data[src_off];
            data[dst_off + 1] = src_data[src_off + 1];
            data[dst_off + 2] = src_data[src_off + 2];
        }
    }

    Ok(Pixbuf::from_bytes(
        &glib::Bytes::from_owned(data),
        gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        width,
        height,
        rowstride,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdk_pixbuf::{Colorspace, Pixbuf};
    use std::fs;

    fn test_pixbuf() -> Pixbuf {
        let width = 8;
        let height = 8;
        let row_stride = width * 3;
        let data = vec![128u8; (height * row_stride) as usize];
        Pixbuf::from_mut_slice(data, Colorspace::Rgb, false, 8, width, height, row_stride)
    }

    #[test]
    fn test_export_png_writes_file() {
        let pixbuf = test_pixbuf();
        let path = std::env::temp_dir().join("screenshot_hero_test_export.png");
        let _ = fs::remove_file(&path);

        export_png(&pixbuf, &path).expect("export_png should succeed");
        assert!(path.exists());
        let len = fs::metadata(&path).unwrap().len();
        assert!(len > 0);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_export_jpeg_writes_file() {
        let pixbuf = test_pixbuf();
        let path = std::env::temp_dir().join("screenshot_hero_test_export.jpg");
        let _ = fs::remove_file(&path);

        export_jpeg(&pixbuf, &path).expect("export_jpeg should succeed");
        assert!(path.exists());
        let len = fs::metadata(&path).unwrap().len();
        assert!(len > 0);

        fs::remove_file(&path).ok();
    }

    fn test_rgba_pixbuf() -> Pixbuf {
        let width = 8;
        let height = 8;
        let row_stride = width * 4;
        let data = vec![128u8; (height * row_stride) as usize];
        Pixbuf::from_mut_slice(data, Colorspace::Rgb, true, 8, width, height, row_stride)
    }

    #[test]
    fn test_export_png_portal_staging_path() {
        let pixbuf = test_pixbuf();
        let path = std::env::temp_dir().join(".xdp-export.png.tmp-unittest");
        let _ = fs::remove_file(&path);

        export_png(&pixbuf, &path).expect("portal staging path export should succeed");
        assert!(path.exists());
        let len = fs::metadata(&path).unwrap().len();
        assert!(len > 0);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_export_jpeg_from_rgba_pixbuf() {
        let pixbuf = test_rgba_pixbuf();
        let path = std::env::temp_dir().join("screenshot_hero_test_export_rgba.jpg");
        let _ = fs::remove_file(&path);

        export_jpeg(&pixbuf, &path).expect("export_jpeg should accept RGBA input");
        assert!(path.exists());
        let len = fs::metadata(&path).unwrap().len();
        assert!(len > 0);

        fs::remove_file(&path).ok();
    }
}
