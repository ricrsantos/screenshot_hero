mod imp;

use gtk::glib;

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
            .build()
    }
}
