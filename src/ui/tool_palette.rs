use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;

use crate::annotations::{ActiveTool, Color};

pub struct ToolPalette {
    container: gtk::Box,
    tool_buttons: Vec<(ActiveTool, gtk::ToggleButton)>,
    tool_changed_cb: Rc<RefCell<Option<Box<dyn Fn(ActiveTool)>>>>,
    color_button: gtk::ColorButton,
    color_changed_cb: Rc<RefCell<Option<Box<dyn Fn(Color)>>>>,
    stroke_spin: gtk::SpinButton,
    stroke_changed_cb: Rc<RefCell<Option<Box<dyn Fn(f32)>>>>,
}

impl ToolPalette {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.set_margin_start(4);
        container.set_margin_end(4);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        // (tool, icon-name, tooltip)
        let tools: [(ActiveTool, &str, &str); 13] = [
            (ActiveTool::Select,       "edit-select-symbolic",          "Select"),
            (ActiveTool::Rectangle,    "draw-rectangle-symbolic",       "Rectangle"),
            (ActiveTool::Ellipse,      "draw-circle-symbolic",          "Ellipse"),
            (ActiveTool::Arrow,        "go-next-symbolic",              "Arrow"),
            (ActiveTool::Line,         "draw-line-symbolic",            "Line"),
            (ActiveTool::Freehand,     "document-edit-symbolic",        "Freehand"),
            (ActiveTool::Text,         "insert-text-symbolic",          "Text"),
            (ActiveTool::Blur,         "zoom-out-symbolic",             "Blur"),
            (ActiveTool::Pixelate,     "view-grid-symbolic",            "Pixelate"),
            (ActiveTool::Redaction,    "edit-clear-symbolic",           "Redaction"),
            (ActiveTool::Timestamp,    "alarm-symbolic",                "Timestamp"),
            (ActiveTool::NumberMarker, "format-list-numbered-symbolic", "Number Marker"),
            (ActiveTool::Callout,      "dialog-information-symbolic",   "Callout"),
        ];

        let mut tool_buttons = Vec::with_capacity(tools.len());
        let mut prev_btn: Option<gtk::ToggleButton> = None;

        for (tool, icon, tooltip) in tools {
            let btn = gtk::ToggleButton::builder()
                .icon_name(icon)
                .tooltip_text(tooltip)
                .build();
            btn.add_css_class("flat");
            if tool == ActiveTool::Select {
                btn.set_active(true);
            }
            if let Some(ref prev) = prev_btn {
                btn.set_group(Some(prev));
            }
            prev_btn = Some(btn.clone());
            tool_buttons.push((tool, btn));
        }

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);

        let color_button = gtk::ColorButton::new();
        color_button.set_rgba(&gdk::RGBA::new(1.0, 0.0, 0.0, 1.0));
        color_button.set_tooltip_text(Some("Stroke Color"));

        let stroke_adjustment = gtk::Adjustment::new(2.0, 1.0, 20.0, 1.0, 1.0, 0.0);
        let stroke_spin = gtk::SpinButton::new(Some(&stroke_adjustment), 1.0, 0);
        stroke_spin.set_tooltip_text(Some("Stroke Width"));

        let tool_changed_cb: Rc<RefCell<Option<Box<dyn Fn(ActiveTool)>>>> =
            Rc::new(RefCell::new(None));
        let color_changed_cb: Rc<RefCell<Option<Box<dyn Fn(Color)>>>> =
            Rc::new(RefCell::new(None));
        let stroke_changed_cb: Rc<RefCell<Option<Box<dyn Fn(f32)>>>> =
            Rc::new(RefCell::new(None));

        for (tool, btn) in &tool_buttons {
            let tool = *tool;
            let cb_cell = Rc::clone(&tool_changed_cb);
            btn.connect_toggled(move |button| {
                if button.is_active() {
                    if let Some(ref cb) = *cb_cell.borrow() {
                        cb(tool);
                    }
                }
            });
            container.append(btn);
        }

        container.append(&separator);
        container.append(&color_button);
        container.append(&stroke_spin);

        let color_cb_cell = Rc::clone(&color_changed_cb);
        color_button.connect_color_set(move |button| {
            let rgba = button.rgba();
            let color = Color {
                r: rgba.red() as f64,
                g: rgba.green() as f64,
                b: rgba.blue() as f64,
                a: rgba.alpha() as f64,
            };
            if let Some(ref cb) = *color_cb_cell.borrow() {
                cb(color);
            }
        });

        let stroke_cb_cell = Rc::clone(&stroke_changed_cb);
        stroke_spin.connect_value_changed(move |spin| {
            let value = spin.value() as f32;
            if let Some(ref cb) = *stroke_cb_cell.borrow() {
                cb(value);
            }
        });

        Self {
            container,
            tool_buttons,
            tool_changed_cb,
            color_button,
            color_changed_cb,
            stroke_spin,
            stroke_changed_cb,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.container.upcast_ref()
    }

    pub fn on_tool_changed(&self, cb: impl Fn(ActiveTool) + 'static) {
        *self.tool_changed_cb.borrow_mut() = Some(Box::new(cb));
    }

    pub fn set_active_tool(&self, tool: ActiveTool) {
        for (active_tool, btn) in &self.tool_buttons {
            btn.set_active(*active_tool == tool);
        }
    }

    pub fn on_color_changed(&self, cb: impl Fn(Color) + 'static) {
        *self.color_changed_cb.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_stroke_changed(&self, cb: impl Fn(f32) + 'static) {
        *self.stroke_changed_cb.borrow_mut() = Some(Box::new(cb));
    }
}
