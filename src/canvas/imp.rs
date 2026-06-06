use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::subclass::prelude::*;

use crate::models::ImageData;

pub struct Canvas {
    pub image: RefCell<Option<ImageData>>,
    pub zoom: Cell<f64>,
    pub pan_offset: Cell<(f64, f64)>,
    pub pointer_pos: Cell<(f64, f64)>,
    pub pan_base: Cell<(f64, f64)>,
    pub zoom_changed_cb: RefCell<Option<Box<dyn Fn(f64)>>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            image: RefCell::new(None),
            zoom: Cell::new(1.0),
            pan_offset: Cell::new((0.0, 0.0)),
            pointer_pos: Cell::new((0.0, 0.0)),
            pan_base: Cell::new((0.0, 0.0)),
            zoom_changed_cb: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Canvas {
    const NAME: &'static str = "ScreenshotHeroCanvas";
    type Type = super::Canvas;
    type ParentType = gtk::DrawingArea;
}

impl ObjectImpl for Canvas {}
impl WidgetImpl for Canvas {}
impl DrawingAreaImpl for Canvas {}
