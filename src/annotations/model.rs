use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub kind: AnnotationKind,
    pub bounds: Rect,
    pub style: AnnotationStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationKind {
    Rectangle,
    Ellipse,
    Arrow(ArrowData),
    Line(ArrowData),
    Freehand(FreehandData),
    Text(TextData),
    Blur,
    Pixelate,
    Redaction,
    Timestamp(TextData),
    NumberMarker(NumberMarkerData),
    Callout(CalloutData),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnnotationStyle {
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArrowData {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreehandData {
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextData {
    pub text: String,
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NumberMarkerData {
    pub number: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalloutData {
    pub text: String,
    pub anchor: Point,
}

impl Rect {
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
}

impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            stroke_color: Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            stroke_width: 2.0,
            fill_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bounds() -> Rect {
        Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        }
    }

    fn sample_annotation(kind: AnnotationKind) -> Annotation {
        Annotation {
            id: Uuid::new_v4(),
            kind,
            bounds: sample_bounds(),
            style: AnnotationStyle::default(),
        }
    }

    #[test]
    fn default_style_is_red_stroke_width_two_no_fill() {
        let style = AnnotationStyle::default();
        assert_eq!(style.stroke_color.r, 1.0);
        assert_eq!(style.stroke_color.g, 0.0);
        assert_eq!(style.stroke_color.b, 0.0);
        assert_eq!(style.stroke_color.a, 1.0);
        assert_eq!(style.stroke_width, 2.0);
        assert!(style.fill_color.is_none());
    }

    #[test]
    fn rectangle_variant_fields_accessible() {
        let ann = sample_annotation(AnnotationKind::Rectangle);
        assert!(matches!(ann.kind, AnnotationKind::Rectangle));
        assert_eq!(ann.bounds.width, 100.0);
    }

    #[test]
    fn ellipse_variant_fields_accessible() {
        let ann = sample_annotation(AnnotationKind::Ellipse);
        assert!(matches!(ann.kind, AnnotationKind::Ellipse));
    }

    #[test]
    fn arrow_variant_fields_accessible() {
        let data = ArrowData {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 50.0, y: 50.0 },
        };
        let ann = sample_annotation(AnnotationKind::Arrow(data));
        if let AnnotationKind::Arrow(d) = ann.kind {
            assert_eq!(d.end.x, 50.0);
        } else {
            panic!("expected Arrow variant");
        }
    }

    #[test]
    fn line_variant_fields_accessible() {
        let data = ArrowData {
            start: Point { x: 1.0, y: 2.0 },
            end: Point { x: 3.0, y: 4.0 },
        };
        let ann = sample_annotation(AnnotationKind::Line(data));
        if let AnnotationKind::Line(d) = ann.kind {
            assert_eq!(d.start.y, 2.0);
        } else {
            panic!("expected Line variant");
        }
    }

    #[test]
    fn freehand_variant_fields_accessible() {
        let data = FreehandData {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 5.0 }],
        };
        let ann = sample_annotation(AnnotationKind::Freehand(data));
        if let AnnotationKind::Freehand(d) = ann.kind {
            assert_eq!(d.points.len(), 2);
        } else {
            panic!("expected Freehand variant");
        }
    }

    #[test]
    fn text_variant_fields_accessible() {
        let data = TextData {
            text: "hello".to_string(),
            font_size: 16.0,
        };
        let ann = sample_annotation(AnnotationKind::Text(data));
        if let AnnotationKind::Text(d) = ann.kind {
            assert_eq!(d.text, "hello");
            assert_eq!(d.font_size, 16.0);
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn blur_variant_fields_accessible() {
        let ann = sample_annotation(AnnotationKind::Blur);
        assert!(matches!(ann.kind, AnnotationKind::Blur));
    }

    #[test]
    fn pixelate_variant_fields_accessible() {
        let ann = sample_annotation(AnnotationKind::Pixelate);
        assert!(matches!(ann.kind, AnnotationKind::Pixelate));
    }

    #[test]
    fn redaction_variant_fields_accessible() {
        let ann = sample_annotation(AnnotationKind::Redaction);
        assert!(matches!(ann.kind, AnnotationKind::Redaction));
    }

    #[test]
    fn timestamp_variant_fields_accessible() {
        let data = TextData {
            text: "2026-06-06".to_string(),
            font_size: 12.0,
        };
        let ann = sample_annotation(AnnotationKind::Timestamp(data));
        if let AnnotationKind::Timestamp(d) = ann.kind {
            assert_eq!(d.text, "2026-06-06");
        } else {
            panic!("expected Timestamp variant");
        }
    }

    #[test]
    fn number_marker_variant_fields_accessible() {
        let data = NumberMarkerData { number: 3 };
        let ann = sample_annotation(AnnotationKind::NumberMarker(data));
        if let AnnotationKind::NumberMarker(d) = ann.kind {
            assert_eq!(d.number, 3);
        } else {
            panic!("expected NumberMarker variant");
        }
    }

    #[test]
    fn callout_variant_fields_accessible() {
        let data = CalloutData {
            text: "note".to_string(),
            anchor: Point { x: 5.0, y: 5.0 },
        };
        let ann = sample_annotation(AnnotationKind::Callout(data));
        if let AnnotationKind::Callout(d) = ann.kind {
            assert_eq!(d.text, "note");
            assert_eq!(d.anchor.x, 5.0);
        } else {
            panic!("expected Callout variant");
        }
    }

    #[test]
    fn rect_contains_point() {
        let rect = Rect {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        assert!(rect.contains(Point { x: 15.0, y: 15.0 }));
        assert!(!rect.contains(Point { x: 5.0, y: 15.0 }));
    }
}
