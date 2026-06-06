use gtk::glib;
use gtk::gio;
use gtk::prelude::ApplicationExtManual;

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, libadwaita::Application;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "com.screenshot_hero.ScreenshotHero")
            .build()
    }

    pub fn run(&self) -> glib::ExitCode {
        ApplicationExtManual::run(self)
    }
}

mod imp {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use libadwaita::subclass::prelude::AdwApplicationImpl;

    use crate::ui::MainWindow;

    #[derive(Default)]
    pub struct Application;

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "ScreenshotHeroApplication";
        type Type = super::Application;
        type ParentType = libadwaita::Application;
    }

    impl ObjectImpl for Application {}
    impl ApplicationImpl for Application {
        fn startup(&self) {
            self.parent_startup();

            let app = self.obj();
            app.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal"]);
            app.set_accels_for_action("win.zoom-out", &["<Control>minus"]);
            app.set_accels_for_action("win.zoom-fit", &["<Control><Shift>f"]);
            app.set_accels_for_action("win.zoom-100", &["<Control>0"]);
            app.set_accels_for_action("win.undo", &["<Control>z"]);
            app.set_accels_for_action("win.redo", &["<Control>y", "<Control><Shift>z"]);
        }

        fn activate(&self) {
            let app = self.obj();
            let window = MainWindow::new(app.as_ref());
            window.present();
        }
    }
    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}
