use std::cell::RefCell;

use gtk::glib;
use gtk::subclass::prelude::*;

use crate::models::ImageData;

#[derive(Default)]
pub struct Canvas {
    pub image: RefCell<Option<ImageData>>,
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
