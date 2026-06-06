use uuid::Uuid;

use super::model::{Annotation, AnnotationKind, AnnotationStyle, Point, Rect};

pub struct AnnotationEngine {
    annotations: Vec<Annotation>,
    selected_id: Option<Uuid>,
    next_number: u32,
}

impl AnnotationEngine {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            selected_id: None,
            next_number: 1,
        }
    }

    pub fn add(&mut self, ann: Annotation) {
        self.annotations.push(ann);
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Annotation> {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            let removed = self.annotations.remove(pos);
            if self.selected_id == Some(id) {
                self.selected_id = None;
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn update_bounds(&mut self, id: Uuid, new_bounds: Rect) {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.bounds = new_bounds;
        }
    }

    pub fn move_to_bounds(&mut self, id: Uuid, from_bounds: Rect, to_bounds: Rect) {
        let dx = to_bounds.x - from_bounds.x;
        let dy = to_bounds.y - from_bounds.y;
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.bounds = to_bounds;
            match &mut ann.kind {
                AnnotationKind::Arrow(data) | AnnotationKind::Line(data) => {
                    data.start.x += dx;
                    data.start.y += dy;
                    data.end.x += dx;
                    data.end.y += dy;
                }
                AnnotationKind::Freehand(data) => {
                    for p in &mut data.points {
                        p.x += dx;
                        p.y += dy;
                    }
                }
                AnnotationKind::Callout(data) => {
                    data.anchor.x += dx;
                    data.anchor.y += dy;
                }
                _ => {}
            }
        }
    }

    pub fn resize_to_bounds(&mut self, id: Uuid, reference_bounds: Rect, new_bounds: Rect) {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.bounds = new_bounds;
            match &mut ann.kind {
                AnnotationKind::Arrow(data) | AnnotationKind::Line(data) => {
                    data.start = scale_point_in_rect(data.start, reference_bounds, new_bounds);
                    data.end = scale_point_in_rect(data.end, reference_bounds, new_bounds);
                }
                AnnotationKind::Freehand(data) => {
                    for p in &mut data.points {
                        *p = scale_point_in_rect(*p, reference_bounds, new_bounds);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn annotation_at(&self, id: Uuid) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    pub fn update_style(&mut self, id: Uuid, new_style: AnnotationStyle) {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.style = new_style;
        }
    }

    pub fn update_text(&mut self, id: Uuid, new_text: String) {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            match &mut ann.kind {
                AnnotationKind::Text(data) | AnnotationKind::Timestamp(data) => {
                    data.text = new_text;
                }
                AnnotationKind::Callout(data) => {
                    data.text = new_text;
                }
                _ => {}
            }
        }
    }

    pub fn select(&mut self, id: Uuid) {
        self.selected_id = Some(id);
    }

    pub fn deselect(&mut self) {
        self.selected_id = None;
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected_id
    }

    pub fn get_selected(&self) -> Option<&Annotation> {
        self.selected_id
            .and_then(|id| self.annotations.iter().find(|a| a.id == id))
    }

    pub fn hit_test(&self, p: Point) -> Option<Uuid> {
        self.annotations
            .iter()
            .rev()
            .find(|ann| ann.bounds.contains(p))
            .map(|ann| ann.id)
    }

    pub fn next_number(&mut self) -> u32 {
        let n = self.next_number;
        self.next_number += 1;
        n
    }

    pub fn reset_number_counter(&mut self) {
        self.next_number = 1;
    }

    pub fn all(&self) -> &[Annotation] {
        &self.annotations
    }
}

impl Default for AnnotationEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn scale_point_in_rect(p: Point, from: Rect, to: Rect) -> Point {
    let rel_x = if from.width > f64::EPSILON {
        (p.x - from.x) / from.width
    } else {
        0.0
    };
    let rel_y = if from.height > f64::EPSILON {
        (p.y - from.y) / from.height
    } else {
        0.0
    };
    Point {
        x: to.x + rel_x * to.width,
        y: to.y + rel_y * to.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::model::{
        AnnotationKind, AnnotationStyle, CalloutData, TextData,
    };

    fn rect_at(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    fn make_annotation(bounds: Rect) -> Annotation {
        Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Rectangle,
            bounds,
            style: AnnotationStyle::default(),
        }
    }

    #[test]
    fn add_and_remove() {
        let mut engine = AnnotationEngine::new();
        let ann = make_annotation(rect_at(0.0, 0.0, 10.0, 10.0));
        let id = ann.id;
        engine.add(ann);
        assert_eq!(engine.all().len(), 1);

        let removed = engine.remove(id).unwrap();
        assert_eq!(removed.id, id);
        assert!(engine.all().is_empty());
        assert!(engine.remove(id).is_none());
    }

    #[test]
    fn hit_test_returns_topmost_overlapping_annotation() {
        let mut engine = AnnotationEngine::new();
        let bottom = make_annotation(rect_at(0.0, 0.0, 100.0, 100.0));
        let top = make_annotation(rect_at(10.0, 10.0, 50.0, 50.0));
        let top_id = top.id;
        engine.add(bottom);
        engine.add(top);

        let hit = engine.hit_test(Point { x: 30.0, y: 30.0 });
        assert_eq!(hit, Some(top_id));
    }

    #[test]
    fn hit_test_returns_none_outside_bounds() {
        let mut engine = AnnotationEngine::new();
        engine.add(make_annotation(rect_at(0.0, 0.0, 10.0, 10.0)));
        assert!(engine.hit_test(Point { x: 50.0, y: 50.0 }).is_none());
    }

    #[test]
    fn select_and_deselect() {
        let mut engine = AnnotationEngine::new();
        let ann = make_annotation(rect_at(0.0, 0.0, 10.0, 10.0));
        let id = ann.id;
        engine.add(ann);

        engine.select(id);
        assert_eq!(engine.selected_id(), Some(id));
        assert_eq!(engine.get_selected().unwrap().id, id);

        engine.deselect();
        assert!(engine.selected_id().is_none());
        assert!(engine.get_selected().is_none());
    }

    #[test]
    fn remove_clears_selection() {
        let mut engine = AnnotationEngine::new();
        let ann = make_annotation(rect_at(0.0, 0.0, 10.0, 10.0));
        let id = ann.id;
        engine.add(ann);
        engine.select(id);
        engine.remove(id);
        assert!(engine.selected_id().is_none());
    }

    #[test]
    fn next_number_increments() {
        let mut engine = AnnotationEngine::new();
        assert_eq!(engine.next_number(), 1);
        assert_eq!(engine.next_number(), 2);
        assert_eq!(engine.next_number(), 3);
    }

    #[test]
    fn reset_number_counter() {
        let mut engine = AnnotationEngine::new();
        engine.next_number();
        engine.next_number();
        engine.reset_number_counter();
        assert_eq!(engine.next_number(), 1);
    }

    #[test]
    fn update_bounds_and_style_and_text() {
        let mut engine = AnnotationEngine::new();
        let ann = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Text(TextData {
                text: "old".to_string(),
                font_size: 16.0,
            }),
            bounds: rect_at(0.0, 0.0, 10.0, 10.0),
            style: AnnotationStyle::default(),
        };
        let id = ann.id;
        engine.add(ann);

        let new_bounds = rect_at(5.0, 5.0, 20.0, 20.0);
        engine.update_bounds(id, new_bounds);
        assert_eq!(engine.all()[0].bounds, new_bounds);

        let new_style = AnnotationStyle {
            stroke_width: 5.0,
            ..AnnotationStyle::default()
        };
        engine.update_style(id, new_style);
        assert_eq!(engine.all()[0].style.stroke_width, 5.0);

        engine.update_text(id, "new".to_string());
        if let AnnotationKind::Text(data) = &engine.all()[0].kind {
            assert_eq!(data.text, "new");
        } else {
            panic!("expected Text variant");
        }

        let callout = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Callout(CalloutData {
                text: "c".to_string(),
                anchor: Point { x: 0.0, y: 0.0 },
            }),
            bounds: rect_at(0.0, 0.0, 10.0, 10.0),
            style: AnnotationStyle::default(),
        };
        let callout_id = callout.id;
        engine.add(callout);
        engine.update_text(callout_id, "updated".to_string());
        if let AnnotationKind::Callout(data) = &engine.all()[1].kind {
            assert_eq!(data.text, "updated");
        } else {
            panic!("expected Callout variant");
        }

        let rect_id = Uuid::new_v4();
        engine.add(Annotation {
            id: rect_id,
            kind: AnnotationKind::Rectangle,
            bounds: rect_at(0.0, 0.0, 10.0, 10.0),
            style: AnnotationStyle::default(),
        });
        engine.update_text(rect_id, "ignored".to_string());
        assert!(matches!(engine.all()[2].kind, AnnotationKind::Rectangle));
    }
}
