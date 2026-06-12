use gdk_pixbuf::Pixbuf;
use gtk::cairo;
use gtk::prelude::*;
use pango::{FontDescription, Weight};
use pangocairo::functions::{create_layout, show_layout};

use crate::annotations::{
    Annotation, AnnotationKind, AnnotationStyle, ArrowData, CalloutData, Color, FreehandData,
    NumberMarkerData, Point, Rect, TextData,
};

const HANDLE_SIZE: f64 = 8.0;

pub fn draw_all(
    cr: &cairo::Context,
    annotations: &[Annotation],
    selected_id: Option<uuid::Uuid>,
    source_pixbuf: Option<&Pixbuf>,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    cr.save().expect("cairo save");
    cr.translate(pan_x, pan_y);
    cr.scale(zoom, zoom);

    for ann in annotations {
        if is_effect(&ann.kind) {
            draw_annotation(cr, ann, source_pixbuf);
        }
    }
    for ann in annotations {
        if is_shape(&ann.kind) {
            draw_annotation(cr, ann, source_pixbuf);
        }
    }
    for ann in annotations {
        if is_text_based(&ann.kind) {
            draw_annotation(cr, ann, source_pixbuf);
        }
    }

    if let Some(id) = selected_id {
        if let Some(ann) = annotations.iter().find(|a| a.id == id) {
            draw_selection_handles(cr, &ann.bounds, zoom);
        }
    }

    cr.restore().expect("cairo restore");
}

pub fn draw_preview(
    cr: &cairo::Context,
    ann: &Annotation,
    source_pixbuf: Option<&Pixbuf>,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) {
    cr.save().expect("cairo save");
    cr.translate(pan_x, pan_y);
    cr.scale(zoom, zoom);

    cr.set_dash(&[4.0 / zoom, 4.0 / zoom], 0.0);
    let mut preview = ann.clone();
    preview.style.stroke_color = Color {
        r: preview.style.stroke_color.r,
        g: preview.style.stroke_color.g,
        b: preview.style.stroke_color.b,
        a: 0.6,
    };
    draw_annotation(cr, &preview, source_pixbuf);

    cr.restore().expect("cairo restore");
}

fn is_effect(kind: &AnnotationKind) -> bool {
    matches!(
        kind,
        AnnotationKind::Blur | AnnotationKind::Pixelate | AnnotationKind::Redaction
    )
}

fn is_shape(kind: &AnnotationKind) -> bool {
    matches!(
        kind,
        AnnotationKind::Rectangle
            | AnnotationKind::Ellipse
            | AnnotationKind::Arrow(_)
            | AnnotationKind::Line(_)
            | AnnotationKind::Freehand(_)
    )
}

fn is_text_based(kind: &AnnotationKind) -> bool {
    matches!(
        kind,
        AnnotationKind::Text(_)
            | AnnotationKind::Timestamp(_)
            | AnnotationKind::NumberMarker(_)
            | AnnotationKind::Callout(_)
    )
}

fn draw_annotation(cr: &cairo::Context, ann: &Annotation, source_pixbuf: Option<&Pixbuf>) {
    match &ann.kind {
        AnnotationKind::Rectangle => draw_rectangle(cr, &ann.bounds, &ann.style),
        AnnotationKind::Ellipse => draw_ellipse(cr, &ann.bounds, &ann.style),
        AnnotationKind::Arrow(data) => draw_arrow(cr, data, &ann.style),
        AnnotationKind::Line(data) => draw_line(cr, data, &ann.style),
        AnnotationKind::Freehand(data) => draw_freehand(cr, data, &ann.style),
        AnnotationKind::Text(data) => draw_text(cr, &ann.bounds, data, &ann.style),
        AnnotationKind::Blur => draw_blur(cr, &ann.bounds, source_pixbuf, &ann.style),
        AnnotationKind::Pixelate => draw_pixelate(cr, &ann.bounds, source_pixbuf, &ann.style),
        AnnotationKind::Redaction => draw_redaction(cr, &ann.bounds, &ann.style),
        AnnotationKind::Timestamp(data) => draw_timestamp(cr, &ann.bounds, data, &ann.style),
        AnnotationKind::NumberMarker(data) => draw_number_marker(cr, &ann.bounds, data, &ann.style),
        AnnotationKind::Callout(data) => draw_callout(cr, &ann.bounds, data, &ann.style),
    }
}

fn apply_stroke(cr: &cairo::Context, style: &AnnotationStyle) {
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.set_line_width(style.stroke_width as f64);
}

fn draw_rectangle(cr: &cairo::Context, bounds: &Rect, style: &AnnotationStyle) {
    apply_stroke(cr, style);
    cr.rectangle(bounds.x, bounds.y, bounds.width, bounds.height);
    let _ = cr.stroke();
}

fn draw_ellipse(cr: &cairo::Context, bounds: &Rect, style: &AnnotationStyle) {
    apply_stroke(cr, style);
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    // Cairo rejects scale(0, …); flat drags (width or height ≈ 0) happen during preview.
    let rx = (bounds.width / 2.0).max(0.5);
    let ry = (bounds.height / 2.0).max(0.5);
    cr.save().expect("cairo save");
    cr.translate(cx, cy);
    cr.scale(rx, ry);
    cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
    cr.restore().expect("cairo restore");
    let _ = cr.stroke();
}

fn draw_line(cr: &cairo::Context, data: &ArrowData, style: &AnnotationStyle) {
    apply_stroke(cr, style);
    cr.move_to(data.start.x, data.start.y);
    cr.line_to(data.end.x, data.end.y);
    let _ = cr.stroke();
}

fn draw_arrow(cr: &cairo::Context, data: &ArrowData, style: &AnnotationStyle) {
    draw_line(cr, data, style);

    let dx = data.end.x - data.start.x;
    let dy = data.end.y - data.start.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        return;
    }

    let ux = dx / len;
    let uy = dy / len;
    let head_len = (12.0_f64).min(len * 0.3);
    let head_width = head_len * 0.5;

    let px = -uy;
    let py = ux;

    let tip = data.end;
    let base = Point {
        x: tip.x - ux * head_len,
        y: tip.y - uy * head_len,
    };
    let left = Point {
        x: base.x + px * head_width,
        y: base.y + py * head_width,
    };
    let right = Point {
        x: base.x - px * head_width,
        y: base.y - py * head_width,
    };

    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.move_to(tip.x, tip.y);
    cr.line_to(left.x, left.y);
    cr.line_to(right.x, right.y);
    cr.close_path();
    let _ = cr.fill();
}

fn draw_freehand(cr: &cairo::Context, data: &FreehandData, style: &AnnotationStyle) {
    if data.points.is_empty() {
        return;
    }
    apply_stroke(cr, style);
    cr.move_to(data.points[0].x, data.points[0].y);
    for p in &data.points[1..] {
        cr.line_to(p.x, p.y);
    }
    let _ = cr.stroke();
}

fn draw_redaction(cr: &cairo::Context, bounds: &Rect, style: &AnnotationStyle) {
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.rectangle(bounds.x, bounds.y, bounds.width, bounds.height);
    let _ = cr.fill();
}

fn draw_effect_placeholder(cr: &cairo::Context, bounds: &Rect, style: &AnnotationStyle) {
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, 0.3);
    cr.rectangle(bounds.x, bounds.y, bounds.width, bounds.height);
    let _ = cr.fill();
}

fn draw_blur(
    cr: &cairo::Context,
    bounds: &Rect,
    source: Option<&Pixbuf>,
    _style: &AnnotationStyle,
) {
    if let Some(pixbuf) = source {
        if let Some(effect) =
            extract_scaled_region(pixbuf, bounds, 8, gdk_pixbuf::InterpType::Bilinear)
        {
            cr.save().expect("cairo save");
            cr.translate(bounds.x, bounds.y);
            cr.set_source_pixbuf(&effect, 0.0, 0.0);
            let _ = cr.paint();
            cr.restore().expect("cairo restore");
            return;
        }
    }
    draw_effect_placeholder(cr, bounds, _style);
}

fn draw_pixelate(
    cr: &cairo::Context,
    bounds: &Rect,
    source: Option<&Pixbuf>,
    style: &AnnotationStyle,
) {
    if let Some(pixbuf) = source {
        if let Some(effect) =
            extract_scaled_region(pixbuf, bounds, 16, gdk_pixbuf::InterpType::Nearest)
        {
            cr.save().expect("cairo save");
            cr.translate(bounds.x, bounds.y);
            cr.set_source_pixbuf(&effect, 0.0, 0.0);
            let _ = cr.paint();
            cr.restore().expect("cairo restore");
            return;
        }
    }
    draw_effect_placeholder(cr, bounds, style);
}

fn extract_scaled_region(
    pixbuf: &Pixbuf,
    bounds: &Rect,
    scale_divisor: i32,
    upscale_filter: gdk_pixbuf::InterpType,
) -> Option<Pixbuf> {
    let x = bounds.x.floor().max(0.0) as i32;
    let y = bounds.y.floor().max(0.0) as i32;
    let w = bounds.width.ceil().max(1.0) as i32;
    let h = bounds.height.ceil().max(1.0) as i32;

    let pw = pixbuf.width();
    let ph = pixbuf.height();
    if x >= pw || y >= ph {
        return None;
    }

    let w = w.min(pw - x);
    let h = h.min(ph - y);
    if w <= 0 || h <= 0 {
        return None;
    }

    let sub = pixbuf.new_subpixbuf(x, y, w, h);
    let small_w = (w / scale_divisor).max(1);
    let small_h = (h / scale_divisor).max(1);
    let small = sub.scale_simple(small_w, small_h, gdk_pixbuf::InterpType::Bilinear)?;
    small.scale_simple(w, h, upscale_filter)
}

fn draw_text(cr: &cairo::Context, bounds: &Rect, data: &TextData, style: &AnnotationStyle) {
    let layout = create_layout(cr);
    layout.set_text(&data.text);
    let font = FontDescription::from_string(&format!("Sans {}px", data.font_size as u32));
    layout.set_font_description(Some(&font));
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.move_to(bounds.x, bounds.y);
    show_layout(cr, &layout);
}

fn draw_timestamp(cr: &cairo::Context, bounds: &Rect, data: &TextData, style: &AnnotationStyle) {
    draw_text(cr, bounds, data, style);
}

fn draw_number_marker(
    cr: &cairo::Context,
    bounds: &Rect,
    data: &NumberMarkerData,
    style: &AnnotationStyle,
) {
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    let radius = bounds.width.min(bounds.height) / 2.0;

    apply_stroke(cr, style);
    cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
    let _ = cr.stroke();

    let layout = create_layout(cr);
    layout.set_text(&data.number.to_string());
    let mut font = FontDescription::from_string("Sans Bold 12px");
    font.set_weight(Weight::Bold);
    layout.set_font_description(Some(&font));

    let (tw, th) = layout.pixel_size();
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.move_to(cx - tw as f64 / 2.0, cy - th as f64 / 2.0);
    show_layout(cr, &layout);
}

fn draw_callout(cr: &cairo::Context, bounds: &Rect, data: &CalloutData, style: &AnnotationStyle) {
    let radius = 6.0_f64;
    apply_stroke(cr, style);

    cr.move_to(bounds.x + radius, bounds.y);
    cr.line_to(bounds.x + bounds.width - radius, bounds.y);
    cr.arc(
        bounds.x + bounds.width - radius,
        bounds.y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    cr.line_to(bounds.x + bounds.width, bounds.y + bounds.height - radius);
    cr.arc(
        bounds.x + bounds.width - radius,
        bounds.y + bounds.height - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.line_to(bounds.x + radius, bounds.y + bounds.height);

    let pointer_base_x = bounds.x + bounds.width * 0.3;
    cr.line_to(pointer_base_x, bounds.y + bounds.height);
    cr.line_to(data.anchor.x, data.anchor.y);
    cr.line_to(pointer_base_x + 20.0, bounds.y + bounds.height);

    cr.line_to(bounds.x + radius, bounds.y + bounds.height);
    cr.arc(
        bounds.x + radius,
        bounds.y + bounds.height - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.line_to(bounds.x, bounds.y + radius);
    cr.arc(
        bounds.x + radius,
        bounds.y + radius,
        radius,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();
    let _ = cr.stroke();

    let layout = create_layout(cr);
    layout.set_text(&data.text);
    let font = FontDescription::from_string("Sans 14px");
    layout.set_font_description(Some(&font));
    let c = style.stroke_color;
    cr.set_source_rgba(c.r, c.g, c.b, c.a);
    cr.move_to(bounds.x + 8.0, bounds.y + 8.0);
    show_layout(cr, &layout);
}

pub fn draw_selection_handles(cr: &cairo::Context, bounds: &Rect, zoom: f64) {
    let half = HANDLE_SIZE / 2.0 / zoom;
    let corners = [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x, bounds.y + bounds.height),
        (bounds.x + bounds.width, bounds.y + bounds.height),
    ];

    // Adwaita blue for better contrast in both themes.
    cr.set_source_rgba(0.21, 0.52, 0.89, 1.0);
    cr.set_line_width(1.0 / zoom);

    for (cx, cy) in corners {
        cr.rectangle(cx - half, cy - half, half * 2.0, half * 2.0);
        let _ = cr.fill();
        let _ = cr.stroke();
    }
}
