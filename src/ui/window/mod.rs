mod imp;

use gtk::glib;
use gtk::subclass::prelude::*;

use crate::canvas::Canvas;
use crate::models::ImageData;
use crate::resources;
use crate::Application;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libadwaita::Window, libadwaita::ApplicationWindow;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .property("title", "Screenshot Hero")
            .property("icon-name", resources::APP_ICON_NAME)
            .build()
    }

    pub fn canvas(&self) -> Canvas {
        self.imp()
            .canvas
            .get()
            .expect("canvas initialized in constructed")
            .clone()
    }

    pub fn set_loaded_image(&self, image: ImageData) {
        let canvas = self.canvas();
        canvas.set_image(image);
        canvas.fit_to_window();
        let enabled = canvas.source_image_path().is_some();
        if let Some(action) = self.imp().save_project_action.get() {
            action.set_enabled(enabled);
        }
        if let Some(action) = self.imp().export_png_action.get() {
            action.set_enabled(enabled);
        }
        if let Some(action) = self.imp().export_jpeg_action.get() {
            action.set_enabled(enabled);
        }
        if let Some(action) = self.imp().copy_to_clipboard_action.get() {
            action.set_enabled(enabled);
        }
    }
}
