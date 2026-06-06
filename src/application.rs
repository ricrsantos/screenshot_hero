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
        fn activate(&self) {
            let app = self.obj();
            let window = MainWindow::new(app.as_ref());
            window.present();
        }
    }
    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}
