use std::sync::OnceLock;

use gtk::gdk;
use gtk::gio;
use gtk::glib;

pub const RESOURCE_BASE_PATH: &str = "/com/screenshot_hero/ScreenshotHero";
pub const TOOL_ICON_DIR: &str = "/com/screenshot_hero/ScreenshotHero/resources/icons";
pub const APP_ICON_THEME_PATH: &str = "/com/screenshot_hero/ScreenshotHero/icons";
pub const APP_ICON_NAME: &str = "com.screenshot_hero.ScreenshotHero";

pub fn register() -> Result<(), String> {
    static REGISTER_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    REGISTER_RESULT
        .get_or_init(|| {
            let bytes = glib::Bytes::from_static(include_bytes!(env!("APP_GRESOURCE_PATH")));
            let resource = gio::Resource::from_data(&bytes)
                .map_err(|err| format!("failed to decode compiled GResource bytes: {err}"))?;
            gio::resources_register(&resource);
            Ok(())
        })
        .clone()
}

pub fn configure_icon_theme() {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path(APP_ICON_THEME_PATH);
}
