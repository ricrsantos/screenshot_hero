use std::cell::OnceCell;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita::prelude::{AdwApplicationWindowExt, AdwDialogExt};
use libadwaita::subclass::application_window::AdwApplicationWindowImpl;
use libadwaita::subclass::window::AdwWindowImpl;

use crate::canvas::Canvas;
use crate::capture::{CaptureError, CaptureService};

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
                        show_capture_error_dialog(&window, &msg);
                    }
                    Err(CaptureError::ImageLoadFailed(msg)) => {
                        log::error!("Screenshot image load failed: {msg}");
                        show_capture_error_dialog(&window, &msg);
                    }
                }
            });
        });
        actions.add_action(&new_screenshot);

        let open_file = gio::SimpleAction::new("open-file", None);
        open_file.connect_activate(|_, _| {});
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

fn show_capture_error_dialog(window: &super::MainWindow, message: &str) {
    let dialog = libadwaita::AlertDialog::new(Some("Screenshot Failed"), Some(message));
    dialog.present(Some(window));
}

impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
impl AdwWindowImpl for MainWindow {}
impl AdwApplicationWindowImpl for MainWindow {}
