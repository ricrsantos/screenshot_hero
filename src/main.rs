use libadwaita::glib;
use screenshot_hero::{Application, LaunchOptions};

fn main() -> glib::ExitCode {
    let launch_options = LaunchOptions::parse(std::env::args());
    Application::new(launch_options.start_with_capture)
        .run_with_args(&launch_options.passthrough_args)
}
