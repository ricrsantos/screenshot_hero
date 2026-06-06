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
    const ZOOM_MIN: f64 = 0.1;
    const ZOOM_MAX: f64 = 8.0;
    const ZOOM_STEP: f64 = 1.25;
    const SCROLL_STEP: f64 = 1.1;

    pub fn new() -> Self {
        let canvas = glib::Object::new::<Self>();

        canvas.set_draw_func(move |widget, cr, width, height| {
            let Some(canvas) = widget.downcast_ref::<Canvas>() else {
                return;
            };
            let imp = canvas.imp();
            let zoom = imp.zoom.get();
            let (pan_x, pan_y) = imp.pan_offset.get();

            if let Some(image) = imp.image.borrow().as_ref() {
                cr.translate(pan_x, pan_y);
                cr.scale(zoom, zoom);
                cr.set_source_pixbuf(image.pixbuf(), 0.0, 0.0);
                cr.source().set_filter(gtk::cairo::Filter::Bilinear);
                let _ = cr.paint();
            } else {
                cr.set_source_rgb(0.12, 0.12, 0.12);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();
            }
        });

        let motion = gtk::EventControllerMotion::new();
        let canvas_weak = canvas.downgrade();
        motion.connect_motion(move |_, x, y| {
            if let Some(c) = canvas_weak.upgrade() {
                c.imp().pointer_pos.set((x, y));
            }
        });
        canvas.add_controller(motion);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let canvas_weak = canvas.downgrade();
        scroll.connect_scroll(move |_, _dx, dy| {
            if let Some(c) = canvas_weak.upgrade() {
                let factor = if dy < 0.0 {
                    Canvas::SCROLL_STEP
                } else {
                    1.0 / Canvas::SCROLL_STEP
                };
                let anchor = c.imp().pointer_pos.get();
                c.apply_zoom(c.imp().zoom.get() * factor, Some(anchor));
            }
            glib::Propagation::Stop
        });
        canvas.add_controller(scroll);

        let drag = gtk::GestureDrag::new();
        drag.set_button(2);
        let canvas_weak_begin = canvas.downgrade();
        drag.connect_drag_begin(move |_, _x, _y| {
            if let Some(c) = canvas_weak_begin.upgrade() {
                c.imp().pan_base.set(c.imp().pan_offset.get());
                c.set_cursor_from_name(Some("grabbing"));
            }
        });
        let canvas_weak_update = canvas.downgrade();
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            if let Some(c) = canvas_weak_update.upgrade() {
                let (base_x, base_y) = c.imp().pan_base.get();
                c.imp()
                    .pan_offset
                    .set((base_x + offset_x, base_y + offset_y));
                c.queue_draw();
            }
        });
        let canvas_weak_end = canvas.downgrade();
        drag.connect_drag_end(move |_, _, _| {
            if let Some(c) = canvas_weak_end.upgrade() {
                c.set_cursor_from_name(None);
            }
        });
        canvas.add_controller(drag);

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

    pub fn zoom_in(&self) {
        self.apply_zoom(self.imp().zoom.get() * Self::ZOOM_STEP, None);
    }

    pub fn zoom_out(&self) {
        self.apply_zoom(self.imp().zoom.get() / Self::ZOOM_STEP, None);
    }

    pub fn zoom_100(&self) {
        let zoom = 1.0;
        let cx = self.width() as f64 / 2.0;
        let cy = self.height() as f64 / 2.0;
        let (iw, ih) = self.image_size();
        let pan_x = cx - iw / 2.0;
        let pan_y = cy - ih / 2.0;
        self.imp().zoom.set(zoom);
        self.imp().pan_offset.set((pan_x, pan_y));
        self.notify_zoom_changed(zoom);
        self.queue_draw();
    }

    pub fn fit_to_window(&self) {
        let cw = self.width() as f64;
        let ch = self.height() as f64;
        if cw <= 0.0 || ch <= 0.0 {
            return;
        }
        let (iw, ih) = self.image_size();
        if iw <= 0.0 || ih <= 0.0 {
            return;
        }
        let zoom = (cw / iw).min(ch / ih).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        let pan_x = (cw - iw * zoom) / 2.0;
        let pan_y = (ch - ih * zoom) / 2.0;
        self.imp().zoom.set(zoom);
        self.imp().pan_offset.set((pan_x, pan_y));
        self.notify_zoom_changed(zoom);
        self.queue_draw();
    }

    pub fn zoom_level(&self) -> f64 {
        self.imp().zoom.get()
    }

    pub fn on_zoom_changed(&self, cb: impl Fn(f64) + 'static) {
        self.imp().zoom_changed_cb.replace(Some(Box::new(cb)));
    }

    fn apply_zoom(&self, raw_zoom: f64, anchor: Option<(f64, f64)>) {
        let old_zoom = self.imp().zoom.get();
        let new_zoom = raw_zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        if (new_zoom - old_zoom).abs() < f64::EPSILON {
            return;
        }

        let (old_pan_x, old_pan_y) = self.imp().pan_offset.get();
        let (ax, ay) = anchor.unwrap_or((self.width() as f64 / 2.0, self.height() as f64 / 2.0));

        let new_pan_x = ax - (ax - old_pan_x) * (new_zoom / old_zoom);
        let new_pan_y = ay - (ay - old_pan_y) * (new_zoom / old_zoom);

        self.imp().zoom.set(new_zoom);
        self.imp().pan_offset.set((new_pan_x, new_pan_y));
        self.notify_zoom_changed(new_zoom);
        self.queue_draw();
    }

    fn image_size(&self) -> (f64, f64) {
        self.imp()
            .image
            .borrow()
            .as_ref()
            .map(|img| (img.width() as f64, img.height() as f64))
            .unwrap_or((0.0, 0.0))
    }

    fn notify_zoom_changed(&self, zoom: f64) {
        if let Some(cb) = self.imp().zoom_changed_cb.borrow().as_ref() {
            cb(zoom);
        }
    }
}
