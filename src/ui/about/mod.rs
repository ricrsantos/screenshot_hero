use gtk::gio;
use gtk::prelude::*;
use libadwaita::prelude::*;

use crate::resources::{self, APP_ICON_NAME};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const WEBSITE_URL: &str = "https://schero.codethings.dev";
const GITHUB_URL: &str = "https://github.com/ricrsantos/screenshot_hero";

pub struct AboutWindow;

impl AboutWindow {
    pub fn show(parent: &impl IsA<gtk::Window>) {
        let window = libadwaita::Window::builder()
            .title("About Screenshot Hero")
            .modal(true)
            .resizable(false)
            .default_width(400)
            .build();
        window.add_css_class("about-dialog");
        window.set_transient_for(Some(parent.upcast_ref()));

        let header = libadwaita::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);

        let close = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Close")
            .css_classes(["flat"])
            .build();
        let window_for_close = window.clone();
        close.connect_clicked(move |_| window_for_close.close());
        header.pack_end(&close);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&header);

        let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
        body.set_vexpand(true);

        let branding = gtk::Box::new(gtk::Orientation::Vertical, 8);
        branding.set_margin_top(8);
        branding.set_margin_bottom(24);
        branding.set_halign(gtk::Align::Center);

        let icon = gtk::Image::from_icon_name(APP_ICON_NAME);
        icon.set_pixel_size(128);
        branding.append(&icon);

        let title = gtk::Label::new(Some("Screenshot Hero"));
        title.add_css_class("title-1");
        branding.append(&title);

        let developer = gtk::Label::new(Some("A Code Things Project"));
        developer.add_css_class("dim-label");
        branding.append(&developer);

        let version = gtk::Label::new(Some(APP_VERSION));
        version.add_css_class("about-version");
        version.set_margin_top(8);
        branding.append(&version);

        body.append(&branding);

        let links = gtk::Box::new(gtk::Orientation::Vertical, 4);
        links.add_css_class("about-links");
        links.set_margin_start(24);
        links.set_margin_end(24);
        links.set_margin_bottom(24);
        links.append(&create_link_row(None, "schero.codethings.dev", WEBSITE_URL));
        links.append(&create_link_row(
            Some("github.svg"),
            "Give us a star",
            GITHUB_URL,
        ));
        body.append(&links);

        content.append(&body);
        window.set_content(Some(&content));
        window.present();
    }
}

pub fn create_header_button() -> gtk::Button {
    let button = gtk::Button::builder()
        .action_name("win.show-about")
        .tooltip_text("About")
        .css_classes(["flat", "image-button"])
        .build();

    let image = gtk::Image::new();
    image.set_pixel_size(20);
    button.set_child(Some(&image));
    resources::set_themed_svg_image(&image, "about.svg", "help-about-symbolic");

    button
}

fn create_link_row(
    icon_file: Option<&str>,
    label: &str,
    uri: &str,
) -> gtk::Button {
    let row = gtk::Button::builder()
        .css_classes(["flat", "about-link-row"])
        .tooltip_text(uri)
        .build();

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.set_margin_top(10);
    hbox.set_margin_bottom(10);

    if let Some(icon_file) = icon_file {
        let icon = gtk::Image::new();
        icon.set_pixel_size(16);
        resources::set_themed_svg_image(&icon, icon_file, "mark-github-symbolic");
        hbox.append(&icon);
    }

    let label_widget = gtk::Label::builder()
        .label(label)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .xalign(0.0)
        .build();
    hbox.append(&label_widget);

    let external = gtk::Image::from_icon_name("adw-external-link-symbolic");
    external.set_pixel_size(16);
    external.add_css_class("dim-label");
    hbox.append(&external);

    row.set_child(Some(&hbox));

    let uri = uri.to_string();
    row.connect_clicked(move |_| {
        if let Err(err) = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
            log::warn!("Failed to open {uri}: {err}");
        }
    });

    row
}

