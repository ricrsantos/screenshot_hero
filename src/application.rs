use gtk::gio;
use gtk::glib;
use gtk::prelude::ApplicationExtManual;
use gtk::subclass::prelude::ObjectSubclassIsExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    pub start_with_capture: bool,
    pub passthrough_args: Vec<String>,
}

impl LaunchOptions {
    pub fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut start_with_capture = false;
        let mut passthrough_args = Vec::new();

        for arg in args.into_iter().map(Into::into) {
            if arg == "--capture" {
                start_with_capture = true;
            } else {
                passthrough_args.push(arg);
            }
        }

        Self {
            start_with_capture,
            passthrough_args,
        }
    }
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, libadwaita::Application;
}

impl Application {
    pub fn new(start_with_capture: bool) -> Self {
        let app: Self = glib::Object::builder()
            .property("application-id", "com.screenshot_hero.ScreenshotHero")
            .property("resource-base-path", crate::resources::RESOURCE_BASE_PATH)
            .build();

        app.imp().start_with_capture.set(start_with_capture);
        app
    }

    pub fn run_with_args(&self, args: &[String]) -> glib::ExitCode {
        ApplicationExtManual::run_with_args(self, args)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new(false)
    }
}

mod imp {
    use std::cell::Cell;

    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use libadwaita::subclass::prelude::AdwApplicationImpl;

    use crate::resources;
    use crate::settings::AppSettings;
    use crate::ui::MainWindow;

    #[derive(Default)]
    pub struct Application {
        pub start_with_capture: Cell<bool>,
    }

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

            if let Err(err) = resources::register() {
                log::warn!("{err}");
            }

            self.parent_startup();
            resources::configure_icon_theme();
            gtk::Window::set_default_icon_name(resources::APP_ICON_NAME);

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
            app.set_accels_for_action("win.new-image", &["<Control>n"]);
            app.set_accels_for_action("win.open-file", &["<Control>o"]);
            app.set_accels_for_action("win.save-project", &["<Control>s"]);
            app.set_accels_for_action("win.copy-to-clipboard", &["<Control>c"]);
            app.set_accels_for_action("win.show-preferences", &["<Control>comma"]);
        }

        fn activate(&self) {
            let app = self.obj();
            let window = MainWindow::new(app.as_ref());
            window.present();

            if self.start_with_capture.replace(false) {
                if let Err(err) = window.activate_action("win.new-screenshot", None) {
                    log::warn!("Unable to trigger startup capture action: {err}");
                }
            }
        }
    }
    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

#[cfg(test)]
mod tests {
    use super::LaunchOptions;

    #[test]
    fn parse_capture_flag_and_keep_other_args() {
        let options =
            LaunchOptions::parse(["screenshot-hero", "--capture", "--gapplication-service"]);

        assert!(options.start_with_capture);
        assert_eq!(
            options.passthrough_args,
            vec![
                "screenshot-hero".to_string(),
                "--gapplication-service".to_string()
            ]
        );
    }

    #[test]
    fn parse_without_capture_flag() {
        let options = LaunchOptions::parse(["screenshot-hero"]);
        assert!(!options.start_with_capture);
        assert_eq!(
            options.passthrough_args,
            vec!["screenshot-hero".to_string()]
        );
    }
}
