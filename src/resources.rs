use std::sync::OnceLock;

use gtk::gdk;
use gtk::gio;
use gtk::glib;

pub const RESOURCE_BASE_PATH: &str = "/dev/codethings/schero";
pub const TOOL_ICON_DIR: &str = "/dev/codethings/schero/resources/icons";
pub const APP_ICON_THEME_PATH: &str = "/dev/codethings/schero/icons";
pub const APP_ICON_NAME: &str = "dev.codethings.schero";

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

pub fn themed_svg_texture(icon_file: &str, dark_mode: bool) -> Option<gdk::Texture> {
    let icon_path = format!("{}/{}", TOOL_ICON_DIR, icon_file);
    let svg_data = gio::resources_lookup_data(&icon_path, gio::ResourceLookupFlags::NONE).ok()?;
    let icon_color = if dark_mode { "#FFFFFF" } else { "#000000" };
    let themed_svg = String::from_utf8_lossy(svg_data.as_ref()).replace("currentColor", icon_color);
    let bytes = glib::Bytes::from_owned(themed_svg.into_bytes());
    gdk::Texture::from_bytes(&bytes).ok()
}

pub fn set_themed_svg_image(image: &gtk::Image, icon_file: &str, fallback_icon_name: &str) {
    let image = image.clone();
    let icon_file = icon_file.to_string();
    let fallback_icon_name = fallback_icon_name.to_string();

    let apply = move || {
        let dark_mode = libadwaita::StyleManager::default().is_dark();
        if let Some(texture) = themed_svg_texture(&icon_file, dark_mode) {
            image.set_paintable(Some(&texture));
        } else {
            image.set_icon_name(Some(&fallback_icon_name));
        }
    };

    apply();
    libadwaita::StyleManager::default().connect_dark_notify(move |_| apply());
}

pub fn configure_icon_theme() {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path(APP_ICON_THEME_PATH);
}
