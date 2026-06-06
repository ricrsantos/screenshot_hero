use uuid::Uuid;

use super::model::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ActiveTool {
    #[default]
    Select,
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Freehand,
    Text,
    Blur,
    Pixelate,
    Redaction,
    Timestamp,
    NumberMarker,
    Callout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandleIndex {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawingState {
    Idle,
    Drawing {
        start: Point,
        current: Point,
    },
    Moving {
        id: Uuid,
        drag_start: Point,
        original_bounds: Rect,
    },
    ResizingHandle {
        id: Uuid,
        handle: HandleIndex,
        original_bounds: Rect,
        drag_start: Point,
    },
    EditingText {
        existing_id: Option<Uuid>,
        position: Point,
    },
}

impl Default for DrawingState {
    fn default() -> Self {
        Self::Idle
    }
}
