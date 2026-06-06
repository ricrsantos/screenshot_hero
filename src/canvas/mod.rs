mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::models::ImageData;

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk::Widget, gtk::DrawingArea;
}

impl Canvas {
    pub fn new() -> Self {
        let canvas = glib::Object::new::<Self>();
        let canvas_ref = canvas.clone();
        canvas.set_draw_func(move |_, cr, width, height| {
            let imp = canvas_ref.imp();
            if let Some(image) = imp.image.borrow().as_ref() {
                cr.set_source_pixbuf(image.pixbuf(), 0.0, 0.0);
                let _ = cr.paint();
            } else {
                cr.set_source_rgb(0.85, 0.85, 0.85);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();
            }
        });
        canvas
    }

    pub fn set_image(&self, image: ImageData) {
        self.imp().image.replace(Some(image));
        self.queue_draw();
    }

    pub fn clear(&self) {
        self.imp().image.replace(None);
        self.queue_draw();
    }
}
