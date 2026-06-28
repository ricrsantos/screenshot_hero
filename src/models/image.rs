use std::path::PathBuf;

pub struct ImageData {
    pixbuf: gdk_pixbuf::Pixbuf,
    source: SourceImage,
}

pub struct SourceImage {
    pub path: PathBuf,
    pub width: i32,
    pub height: i32,
}

impl ImageData {
    pub fn from_pixbuf(pixbuf: gdk_pixbuf::Pixbuf, source: SourceImage) -> Self {
        Self { pixbuf, source }
    }

    pub fn pixbuf(&self) -> &gdk_pixbuf::Pixbuf {
        &self.pixbuf
    }

    pub fn source(&self) -> &SourceImage {
        &self.source
    }

    pub fn width(&self) -> i32 {
        self.pixbuf.width()
    }

    pub fn height(&self) -> i32 {
        self.pixbuf.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdk_pixbuf::{Colorspace, Pixbuf};

    fn test_pixbuf(width: i32, height: i32) -> Pixbuf {
        let row_stride = width * 3;
        let data = vec![0u8; (height * row_stride) as usize];
        Pixbuf::from_mut_slice(data, Colorspace::Rgb, false, 8, width, height, row_stride)
    }

    #[test]
    fn test_width_height_accessors_match_pixbuf_dimensions() {
        let width = 64;
        let height = 48;
        let pixbuf = test_pixbuf(width, height);
        let source = SourceImage {
            path: PathBuf::from("/tmp/test.png"),
            width,
            height,
        };
        let image = ImageData::from_pixbuf(pixbuf, source);

        assert_eq!(image.width(), width);
        assert_eq!(image.height(), height);
    }
}
