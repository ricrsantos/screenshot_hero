use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::cairo::{self, Format};
use gtk::prelude::*;

use crate::annotations::Annotation;
use crate::canvas::renderer::draw_all;

pub fn render_to_pixbuf(source: &Pixbuf, annotations: &[Annotation]) -> Option<Pixbuf> {
    let width = source.width();
    let height = source.height();
    if width <= 0 || height <= 0 {
        return None;
    }

    let surface = cairo::ImageSurface::create(Format::ARgb32, width, height).ok()?;
    let cr = cairo::Context::new(&surface).ok()?;

    cr.set_source_pixbuf(source, 0.0, 0.0);
    let _ = cr.paint();

    draw_all(
        &cr,
        annotations,
        None,
        Some(source),
        1.0,
        0.0,
        0.0,
    );

    surface_to_pixbuf(&surface)
}

fn surface_to_pixbuf(surface: &cairo::ImageSurface) -> Option<Pixbuf> {
    let width = surface.width();
    let height = surface.height();
    if width <= 0 || height <= 0 {
        return None;
    }

    let rowstride = width * 4;
    let mut pixels = vec![0u8; (height * rowstride) as usize];

    surface
        .with_data(|data| {
            let src_stride = surface.stride() as usize;
            let dst_stride = rowstride as usize;

            for y in 0..height as usize {
                for x in 0..width as usize {
                    let src_off = y * src_stride + x * 4;
                    let b = data[src_off];
                    let g = data[src_off + 1];
                    let r = data[src_off + 2];
                    let a = data[src_off + 3];

                    let (r, g, b) = unpremultiply(r, g, b, a);

                    let dst_off = y * dst_stride + x * 4;
                    pixels[dst_off] = r;
                    pixels[dst_off + 1] = g;
                    pixels[dst_off + 2] = b;
                    pixels[dst_off + 3] = a;
                }
            }
        })
        .ok()?;

    let bytes = glib::Bytes::from_owned(pixels);
    Some(Pixbuf::from_bytes(
        &bytes,
        Colorspace::Rgb,
        true,
        8,
        width,
        height,
        rowstride,
    ))
}

fn unpremultiply(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8) {
    if a == 0 {
        (0, 0, 0)
    } else if a == 255 {
        (r, g, b)
    } else {
        let a_f = a as u32;
        (
            ((r as u32 * 255 + a_f / 2) / a_f) as u8,
            ((g as u32 * 255 + a_f / 2) / a_f) as u8,
            ((b as u32 * 255 + a_f / 2) / a_f) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdk_pixbuf::Pixbuf;

    fn test_pixbuf(width: i32, height: i32) -> Pixbuf {
        let row_stride = width * 3;
        let data = vec![0u8; (height * row_stride) as usize];
        Pixbuf::from_mut_slice(data, Colorspace::Rgb, false, 8, width, height, row_stride)
    }

    #[test]
    fn test_render_to_pixbuf_dimensions() {
        let source = test_pixbuf(100, 80);
        let result = render_to_pixbuf(&source, &[]).expect("render should succeed");
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 80);
    }

    #[test]
    fn test_render_flat_ellipse_does_not_panic() {
        use crate::annotations::{
            Annotation, AnnotationKind, AnnotationStyle, Color, Rect,
        };
        use uuid::Uuid;

        let source = test_pixbuf(100, 80);
        let ann = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Ellipse,
            bounds: Rect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 0.0,
            },
            style: AnnotationStyle {
                stroke_color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                stroke_width: 2.0,
                fill_color: None,
            },
        };
        render_to_pixbuf(&source, &[ann]).expect("flat ellipse should render");
    }
}
