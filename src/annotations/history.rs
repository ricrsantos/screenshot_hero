use uuid::Uuid;

use super::engine::AnnotationEngine;
use super::model::{Annotation, AnnotationStyle, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationCommand {
    Add(Annotation),
    Remove(Annotation),
    UpdateBounds {
        id: Uuid,
        old_bounds: Rect,
        new_bounds: Rect,
    },
    UpdateStyle {
        id: Uuid,
        old_style: AnnotationStyle,
        new_style: AnnotationStyle,
    },
    UpdateText {
        id: Uuid,
        old_text: String,
        new_text: String,
    },
}

pub struct History {
    undo_stack: Vec<AnnotationCommand>,
    redo_stack: Vec<AnnotationCommand>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: AnnotationCommand) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, engine: &mut AnnotationEngine) -> bool {
        let Some(cmd) = self.undo_stack.pop() else {
            return false;
        };
        let inverse = inverse_command(&cmd);
        apply_command(engine, &inverse);
        self.redo_stack.push(inverse);
        true
    }

    pub fn redo(&mut self, engine: &mut AnnotationEngine) -> bool {
        let Some(cmd) = self.redo_stack.pop() else {
            return false;
        };
        let inverse = inverse_command(&cmd);
        apply_command(engine, &inverse);
        self.undo_stack.push(inverse);
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

fn apply_command(engine: &mut AnnotationEngine, cmd: &AnnotationCommand) {
    match cmd {
        AnnotationCommand::Add(ann) => {
            engine.add(ann.clone());
        }
        AnnotationCommand::Remove(ann) => {
            engine.remove(ann.id);
        }
        AnnotationCommand::UpdateBounds {
            id,
            old_bounds,
            new_bounds,
        } => {
            engine.resize_to_bounds(*id, *old_bounds, *new_bounds);
        }
        AnnotationCommand::UpdateStyle { id, new_style, .. } => {
            engine.update_style(*id, *new_style);
        }
        AnnotationCommand::UpdateText { id, new_text, .. } => {
            engine.update_text(*id, new_text.clone());
        }
    }
}

fn inverse_command(cmd: &AnnotationCommand) -> AnnotationCommand {
    match cmd {
        AnnotationCommand::Add(ann) => AnnotationCommand::Remove(ann.clone()),
        AnnotationCommand::Remove(ann) => AnnotationCommand::Add(ann.clone()),
        AnnotationCommand::UpdateBounds {
            id,
            old_bounds,
            new_bounds,
        } => AnnotationCommand::UpdateBounds {
            id: *id,
            old_bounds: *new_bounds,
            new_bounds: *old_bounds,
        },
        AnnotationCommand::UpdateStyle {
            id,
            old_style,
            new_style,
        } => AnnotationCommand::UpdateStyle {
            id: *id,
            old_style: *new_style,
            new_style: *old_style,
        },
        AnnotationCommand::UpdateText {
            id,
            old_text,
            new_text,
        } => AnnotationCommand::UpdateText {
            id: *id,
            old_text: new_text.clone(),
            new_text: old_text.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::model::{AnnotationKind, AnnotationStyle, TextData};

    fn sample_annotation() -> Annotation {
        Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Rectangle,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            style: AnnotationStyle::default(),
        }
    }

    #[test]
    fn push_undo_redo_cycle() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        let ann = sample_annotation();
        let id = ann.id;

        history.push(AnnotationCommand::Add(ann.clone()));
        apply_command(&mut engine, &AnnotationCommand::Add(ann));
        assert_eq!(engine.all().len(), 1);

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert!(history.undo(&mut engine));
        assert!(engine.all().is_empty());
        assert!(history.can_redo());

        assert!(history.redo(&mut engine));
        assert_eq!(engine.all().len(), 1);
        assert_eq!(engine.all()[0].id, id);
    }

    #[test]
    fn undo_past_empty_returns_false() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        assert!(!history.undo(&mut engine));
        assert!(!history.redo(&mut engine));
    }

    #[test]
    fn new_push_clears_redo_stack() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        let ann = sample_annotation();

        history.push(AnnotationCommand::Add(ann.clone()));
        apply_command(&mut engine, &AnnotationCommand::Add(ann.clone()));
        history.undo(&mut engine);
        assert!(history.can_redo());

        let ann2 = sample_annotation();
        history.push(AnnotationCommand::Add(ann2));
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_update_bounds_restores_old_bounds() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        let ann = sample_annotation();
        let id = ann.id;
        let old_bounds = ann.bounds;
        let new_bounds = Rect {
            x: 5.0,
            y: 5.0,
            width: 20.0,
            height: 20.0,
        };

        engine.add(ann);
        engine.update_bounds(id, new_bounds);
        history.push(AnnotationCommand::UpdateBounds {
            id,
            old_bounds,
            new_bounds,
        });

        history.undo(&mut engine);
        assert_eq!(engine.all()[0].bounds, old_bounds);

        history.redo(&mut engine);
        assert_eq!(engine.all()[0].bounds, new_bounds);
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        let ann1 = sample_annotation();
        let ann2 = sample_annotation();

        history.push(AnnotationCommand::Add(ann1.clone()));
        apply_command(&mut engine, &AnnotationCommand::Add(ann1));
        history.push(AnnotationCommand::Add(ann2.clone()));
        apply_command(&mut engine, &AnnotationCommand::Add(ann2));
        history.undo(&mut engine);
        assert!(history.can_undo());
        assert!(history.can_redo());

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_update_text_restores_old_text() {
        let mut engine = AnnotationEngine::new();
        let mut history = History::new();
        let ann = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Text(TextData {
                text: "old".to_string(),
                font_size: 16.0,
            }),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            style: AnnotationStyle::default(),
        };
        let id = ann.id;

        engine.add(ann);
        engine.update_text(id, "new".to_string());
        history.push(AnnotationCommand::UpdateText {
            id,
            old_text: "old".to_string(),
            new_text: "new".to_string(),
        });

        history.undo(&mut engine);
        if let AnnotationKind::Text(data) = &engine.all()[0].kind {
            assert_eq!(data.text, "old");
        } else {
            panic!("expected Text variant");
        }
    }
}
