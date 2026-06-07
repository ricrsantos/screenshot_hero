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

    use crate::settings::AppSettings;
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
            let _ = env_logger::Builder::new()
                .filter_level(log::LevelFilter::Trace)
                .try_init();

            self.parent_startup();

            if let Some(settings) = AppSettings::try_new() {
                log::set_max_level(settings.log_level());
                libadwaita::StyleManager::default()
                    .set_color_scheme(settings.color_scheme().to_adw_color_scheme());

                settings.connect_changed({
                    let settings = settings.clone();
                    move |key| match key {
                        "log-level" => log::set_max_level(settings.log_level()),
                        "color-scheme" => libadwaita::StyleManager::default()
                            .set_color_scheme(settings.color_scheme().to_adw_color_scheme()),
                        _ => {}
                    }
                });
            } else {
                log::warn!("GSettings schema unavailable; using built-in defaults");
            }

            let app = self.obj();
            app.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal"]);
            app.set_accels_for_action("win.zoom-out", &["<Control>minus"]);
            app.set_accels_for_action("win.zoom-fit", &["<Control><Shift>f"]);
            app.set_accels_for_action("win.zoom-100", &["<Control>0"]);
            app.set_accels_for_action("win.undo", &["<Control>z"]);
            app.set_accels_for_action("win.redo", &["<Control>y", "<Control><Shift>z"]);
            app.set_accels_for_action("win.save-project", &["<Control>s"]);
            app.set_accels_for_action("win.show-preferences", &["<Control>comma"]);
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
