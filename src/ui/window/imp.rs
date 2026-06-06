use std::cell::OnceCell;
use std::path::Path;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita::prelude::AdwApplicationWindowExt;

use crate::ui::dialogs::show_error_dialog;
use libadwaita::subclass::application_window::AdwApplicationWindowImpl;
use libadwaita::subclass::window::AdwWindowImpl;

use crate::canvas::Canvas;
use crate::capture::{CaptureError, CaptureService, FileLoader, LoadError};

#[derive(Default)]
pub struct MainWindow {
    pub(crate) canvas: OnceCell<Canvas>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindow {
    const NAME: &'static str = "ScreenshotHeroMainWindow";
    type Type = super::MainWindow;
    type ParentType = libadwaita::ApplicationWindow;
}

impl ObjectImpl for MainWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let window = self.obj();

        let canvas = Canvas::new();
        self.canvas
            .set(canvas.clone())
            .expect("canvas initialized once");

        let actions = gio::SimpleActionGroup::new();

        let new_screenshot = gio::SimpleAction::new("new-screenshot", None);
        let window_for_capture = window.clone();
        let canvas_for_capture = canvas.clone();
        new_screenshot.connect_activate(move |_, _| {
            let window = window_for_capture.clone();
            let canvas = canvas_for_capture.clone();
            glib::spawn_future_local(async move {
                match CaptureService::capture().await {
                    Ok(Some(image)) => canvas.set_image(image),
                    Ok(None) | Err(CaptureError::PortalCancelled) => {}
                    Err(CaptureError::PortalUnavailable(msg)) => {
                        log::error!("Screenshot portal unavailable: {msg}");
                        show_error_dialog(&window, "Screenshot Failed", &msg);
                    }
                    Err(CaptureError::ImageLoadFailed(msg)) => {
                        log::error!("Screenshot image load failed: {msg}");
                        show_error_dialog(&window, "Screenshot Failed", &msg);
                    }
                }
            });
        });
        actions.add_action(&new_screenshot);

        let open_file = gio::SimpleAction::new("open-file", None);
        let window_for_open = window.clone();
        let canvas_for_open = canvas.clone();
        open_file.connect_activate(move |_, _| {
            let window = window_for_open.clone();
            let canvas = canvas_for_open.clone();
            glib::spawn_future_local(async move {
                let filter = gtk::FileFilter::new();
                filter.set_name(Some("PNG and JPEG Images"));
                filter.add_mime_type("image/png");
                filter.add_mime_type("image/jpeg");
                filter.add_pattern("*.png");
                filter.add_pattern("*.jpg");
                filter.add_pattern("*.jpeg");

                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);

                let dialog = gtk::FileDialog::new();
                dialog.set_title("Open Image");
                dialog.set_modal(true);
                dialog.set_filters(Some(&filters));
                dialog.set_default_filter(Some(&filter));

                let file = match dialog.open_future(Some(&window)).await {
                    Ok(file) => file,
                    Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => return,
                    Err(error) => {
                        log::error!("File dialog failed: {error}");
                        return;
                    }
                };

                let Some(path) = file.path() else {
                    let message = "Selected file has no local path";
                    log::error!("{message}");
                    show_error_dialog(&window, "Open Failed", message);
                    return;
                };

                match FileLoader::load_from_path(&path) {
                    Ok(image) => canvas.set_image(image),
                    Err(error) => {
                        let message = format_load_error(&path, &error);
                        log::error!("Failed to load image: {message}");
                        show_error_dialog(&window, "Open Failed", &message);
                    }
                }
            });
        });
        actions.add_action(&open_file);

        window.insert_action_group("win", Some(&actions));

        let header = libadwaita::HeaderBar::new();

        let new_button = gtk::Button::builder()
            .label("New Screenshot")
            .action_name("win.new-screenshot")
            .build();
        header.pack_start(&new_button);

        let open_button = gtk::Button::builder()
            .label("Open File")
            .action_name("win.open-file")
            .build();
        header.pack_start(&open_button);

        let toolbar_view = libadwaita::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&canvas));

        window.set_content(Some(&toolbar_view));
    }
}

fn format_load_error(path: &Path, error: &LoadError) -> String {
    let reason = match error {
        LoadError::FileNotFound(_) => "file not found".to_string(),
        LoadError::UnsupportedFormat(_) => "unsupported format".to_string(),
        LoadError::DecodeFailed(message) => message.clone(),
        LoadError::InvalidUri(message) => message.clone(),
    };

    format!("{}: {reason}", path.display())
}

impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
impl AdwWindowImpl for MainWindow {}
impl AdwApplicationWindowImpl for MainWindow {}
