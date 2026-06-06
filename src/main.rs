use libadwaita::glib;
use screenshot_hero::Application;

fn main() -> glib::ExitCode {
    Application::new().run()
}
