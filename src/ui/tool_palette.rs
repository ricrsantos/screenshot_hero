use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::annotations::{ActiveTool, Color};
use crate::resources;

pub struct ToolPalette {
    container: gtk::Box,
    tool_changed_cb: Rc<RefCell<Option<Box<dyn Fn(ActiveTool)>>>>,
    color_changed_cb: Rc<RefCell<Option<Box<dyn Fn(Color)>>>>,
    stroke_changed_cb: Rc<RefCell<Option<Box<dyn Fn(f32)>>>>,
}

// Tool groups — separators are inserted between groups.
const TOOLS: &[(ActiveTool, &str, &str)] = &[
    // ── Navigation ─────────────────────────────────────────
    (
        ActiveTool::Crop,
        "crop.svg",
        "Crop (double click to apply, Esc to cancel)",
    ),
    (ActiveTool::Pan, "pan.svg", "Pan"),
    // ── Selection ──────────────────────────────────────────
    (ActiveTool::Select,       "select.svg",          "Select"),
    // ── Shapes ─────────────────────────────────────────────
    (ActiveTool::Rectangle,    "rectangle.svg",  "Rectangle"),
    (ActiveTool::Ellipse,      "ellipse.svg",         "Ellipse"),
    (ActiveTool::Arrow,        "arrow.svg",              "Arrow"),
    (ActiveTool::Line,         "line.svg","Line"),
    // ── Drawing ────────────────────────────────────────────
    (ActiveTool::Freehand,     "freehand.svg",        "Freehand"),
    (ActiveTool::Text,         "text.svg",          "Text"),
    // ── Effects ────────────────────────────────────────────
    (ActiveTool::Blur,         "blur.svg",             "Blur"),
    (ActiveTool::Pixelate,     "pixelate.svg",            "Pixelate"),
    (ActiveTool::Redaction,    "redaction.svg",           "Redaction"),
    // ── Labels ─────────────────────────────────────────────
    (ActiveTool::Timestamp,    "timestamp.svg",                "Timestamp"),
    (ActiveTool::NumberMarker, "number-marker.svg",  "Number Marker"),
    (ActiveTool::Callout,      "callout.svg",   "Callout"),
];

// Indices (0-based) after which a separator is inserted.
const SEP_AFTER: &[usize] = &[1, 6, 8, 11];

impl ToolPalette {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_margin_start(4);
        container.set_margin_end(4);
        container.set_margin_top(6);
        container.set_margin_bottom(6);
        container.add_css_class("tool-palette");

        let mut tool_buttons = Vec::with_capacity(TOOLS.len());
        let mut tool_images = Vec::with_capacity(TOOLS.len());
        let mut prev_btn: Option<gtk::ToggleButton> = None;

        for (idx, &(tool, icon_file, tooltip)) in TOOLS.iter().enumerate() {
            let (btn, image) = build_tool_button(tooltip);
            btn.add_css_class("flat");
            btn.add_css_class("tool-btn");
            if tool == ActiveTool::Select {
                btn.set_active(true);
            }
            if let Some(ref prev) = prev_btn {
                btn.set_group(Some(prev));
            }
            prev_btn = Some(btn.clone());
            container.append(&btn);
            tool_buttons.push((tool, btn));
            tool_images.push((icon_file.to_string(), image));

            if SEP_AFTER.contains(&idx) {
                let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
                sep.add_css_class("tool-sep");
                container.append(&sep);
            }
        }

        let style_sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        container.append(&style_sep);

        let color_button = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
        color_button.set_rgba(&gdk::RGBA::new(1.0, 0.0, 0.0, 1.0));
        color_button.set_tooltip_text(Some("Stroke Color"));
        color_button.add_css_class("tool-color");
        color_button.set_width_request(18);
        color_button.set_height_request(18);
        container.append(&color_button);

        let stroke_controls = gtk::Box::new(gtk::Orientation::Vertical, 2);
        stroke_controls.add_css_class("tool-stroke");
        stroke_controls.set_tooltip_text(Some("Stroke Width"));

        let stroke_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        stroke_buttons.set_homogeneous(true);
        stroke_buttons.set_width_request(36);

        let stroke_decrease = gtk::Button::with_label("-");
        stroke_decrease.add_css_class("flat");
        stroke_decrease.add_css_class("tool-step-btn");

        let stroke_increase = gtk::Button::with_label("+");
        stroke_increase.add_css_class("flat");
        stroke_increase.add_css_class("tool-step-btn");

        stroke_buttons.append(&stroke_decrease);
        stroke_buttons.append(&stroke_increase);
        stroke_controls.append(&stroke_buttons);

        let stroke_value_label = gtk::Label::new(None);
        stroke_value_label.add_css_class("tool-stroke-value");
        stroke_controls.append(&stroke_value_label);
        container.append(&stroke_controls);

        let tool_changed_cb: Rc<RefCell<Option<Box<dyn Fn(ActiveTool)>>>> =
            Rc::new(RefCell::new(None));
        let color_changed_cb: Rc<RefCell<Option<Box<dyn Fn(Color)>>>> =
            Rc::new(RefCell::new(None));
        let stroke_changed_cb: Rc<RefCell<Option<Box<dyn Fn(f32)>>>> =
            Rc::new(RefCell::new(None));

        let style_manager = libadwaita::StyleManager::default();
        let tool_images = Rc::new(tool_images);
        apply_tool_icon_theme(tool_images.as_ref(), style_manager.is_dark());
        {
            let tool_images = Rc::clone(&tool_images);
            style_manager.connect_dark_notify(move |manager| {
                apply_tool_icon_theme(tool_images.as_ref(), manager.is_dark());
            });
        }

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
        }

        let color_cb_cell = Rc::clone(&color_changed_cb);
        color_button.connect_rgba_notify(move |button| {
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

        let stroke_value = Rc::new(RefCell::new(2.0_f32));
        set_stroke_value(&stroke_value, &stroke_value_label, &stroke_changed_cb, 2.0);

        let stroke_value_dec = Rc::clone(&stroke_value);
        let stroke_label_dec = stroke_value_label.clone();
        let stroke_cb_dec = Rc::clone(&stroke_changed_cb);
        stroke_decrease.connect_clicked(move |_| {
            let current = *stroke_value_dec.borrow();
            set_stroke_value(
                &stroke_value_dec,
                &stroke_label_dec,
                &stroke_cb_dec,
                (current - 1.0).clamp(1.0, 20.0),
            );
        });

        let stroke_value_inc = Rc::clone(&stroke_value);
        let stroke_label_inc = stroke_value_label.clone();
        let stroke_cb_inc = Rc::clone(&stroke_changed_cb);
        stroke_increase.connect_clicked(move |_| {
            let current = *stroke_value_inc.borrow();
            set_stroke_value(
                &stroke_value_inc,
                &stroke_label_inc,
                &stroke_cb_inc,
                (current + 1.0).clamp(1.0, 20.0),
            );
        });

        Self {
            container,
            tool_changed_cb,
            color_changed_cb,
            stroke_changed_cb,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.container.upcast_ref()
    }

    pub fn on_tool_changed(&self, cb: impl Fn(ActiveTool) + 'static) {
        *self.tool_changed_cb.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_color_changed(&self, cb: impl Fn(Color) + 'static) {
        *self.color_changed_cb.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_stroke_changed(&self, cb: impl Fn(f32) + 'static) {
        *self.stroke_changed_cb.borrow_mut() = Some(Box::new(cb));
    }
}

fn build_tool_button(tooltip: &str) -> (gtk::ToggleButton, gtk::Image) {
    let image = gtk::Image::new();
    image.set_pixel_size(20);
    image.set_icon_size(gtk::IconSize::Normal);

    let btn = gtk::ToggleButton::builder().tooltip_text(tooltip).build();
    btn.set_child(Some(&image));
    (btn, image)
}

fn apply_tool_icon_theme(tool_images: &[(String, gtk::Image)], dark_mode: bool) {
    for (icon_file, image) in tool_images {
        if let Some(texture) = themed_svg_texture(icon_file, dark_mode) {
            image.set_paintable(Some(&texture));
        }
    }
}

fn themed_svg_texture(icon_file: &str, dark_mode: bool) -> Option<gdk::Texture> {
    let icon_path = format!("{}/{}", resources::TOOL_ICON_DIR, icon_file);
    let svg_data = gio::resources_lookup_data(&icon_path, gio::ResourceLookupFlags::NONE).ok()?;
    let icon_color = if dark_mode { "#FFFFFF" } else { "#000000" };
    let themed_svg = String::from_utf8_lossy(svg_data.as_ref()).replace("currentColor", icon_color);
    let bytes = glib::Bytes::from_owned(themed_svg.into_bytes());
    gdk::Texture::from_bytes(&bytes).ok()
}

fn set_stroke_value(
    stroke_value: &Rc<RefCell<f32>>,
    stroke_label: &gtk::Label,
    stroke_changed_cb: &Rc<RefCell<Option<Box<dyn Fn(f32)>>>>,
    value: f32,
) {
    *stroke_value.borrow_mut() = value;
    stroke_label.set_text(&format!("{}", value.round() as i32));
    if let Some(ref cb) = *stroke_changed_cb.borrow() {
        cb(value);
    }
}
