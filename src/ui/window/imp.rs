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
use crate::ui::ToolPalette;

#[derive(Default)]
pub struct MainWindow {
    pub(crate) canvas: OnceCell<Canvas>,
    tool_palette: OnceCell<ToolPalette>,
    zoom_label: OnceCell<gtk::Label>,
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

        let zoom_label = gtk::Label::new(Some("100%"));
        self.zoom_label
            .set(zoom_label.clone())
            .expect("zoom_label initialized once");

        let zoom_label_for_cb = zoom_label.clone();
        canvas.on_zoom_changed(move |zoom| {
            zoom_label_for_cb.set_text(&format!("{}%", (zoom * 100.0).round() as i32));
        });

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

        let zoom_in = gio::SimpleAction::new("zoom-in", None);
        let canvas_for_zoom_in = canvas.clone();
        zoom_in.connect_activate(move |_, _| {
            canvas_for_zoom_in.zoom_in();
        });
        actions.add_action(&zoom_in);

        let zoom_out = gio::SimpleAction::new("zoom-out", None);
        let canvas_for_zoom_out = canvas.clone();
        zoom_out.connect_activate(move |_, _| {
            canvas_for_zoom_out.zoom_out();
        });
        actions.add_action(&zoom_out);

        let zoom_fit = gio::SimpleAction::new("zoom-fit", None);
        let canvas_for_zoom_fit = canvas.clone();
        zoom_fit.connect_activate(move |_, _| {
            canvas_for_zoom_fit.fit_to_window();
        });
        actions.add_action(&zoom_fit);

        let zoom_100 = gio::SimpleAction::new("zoom-100", None);
        let canvas_for_zoom_100 = canvas.clone();
        zoom_100.connect_activate(move |_, _| {
            canvas_for_zoom_100.zoom_100();
        });
        actions.add_action(&zoom_100);

        let undo = gio::SimpleAction::new("undo", None);
        undo.set_enabled(false);
        let canvas_for_undo = canvas.clone();
        undo.connect_activate(move |_, _| {
            canvas_for_undo.undo();
        });
        actions.add_action(&undo);

        let redo = gio::SimpleAction::new("redo", None);
        redo.set_enabled(false);
        let canvas_for_redo = canvas.clone();
        redo.connect_activate(move |_, _| {
            canvas_for_redo.redo();
        });
        actions.add_action(&redo);

        let undo_for_cb = undo.clone();
        let redo_for_cb = redo.clone();
        let canvas_for_annotation_cb = canvas.clone();
        canvas.on_annotation_changed(move || {
            undo_for_cb.set_enabled(canvas_for_annotation_cb.can_undo());
            redo_for_cb.set_enabled(canvas_for_annotation_cb.can_redo());
        });

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

        let zoom_in_button = gtk::Button::builder()
            .label("+")
            .action_name("win.zoom-in")
            .build();
        let zoom_out_button = gtk::Button::builder()
            .label("-")
            .action_name("win.zoom-out")
            .build();
        let zoom_fit_button = gtk::Button::builder()
            .label("fit")
            .action_name("win.zoom-fit")
            .build();
        let zoom_100_button = gtk::Button::builder()
            .label("1:1")
            .action_name("win.zoom-100")
            .build();

        // pack_end: first = nearest title, last = outermost right → [label][1:1][fit][-][+]
        header.pack_end(&zoom_label);
        header.pack_end(&zoom_100_button);
        header.pack_end(&zoom_fit_button);
        header.pack_end(&zoom_out_button);
        header.pack_end(&zoom_in_button);

        let tool_palette = ToolPalette::new();

        let canvas_for_tool = canvas.clone();
        tool_palette.on_tool_changed(move |tool| {
            canvas_for_tool.set_active_tool(tool);
        });

        let canvas_for_color = canvas.clone();
        tool_palette.on_color_changed(move |color| {
            let mut style = canvas_for_color.current_style();
            style.stroke_color = color;
            canvas_for_color.set_current_style(style);
        });

        let canvas_for_stroke = canvas.clone();
        tool_palette.on_stroke_changed(move |width| {
            let mut style = canvas_for_stroke.current_style();
            style.stroke_width = width;
            canvas_for_stroke.set_current_style(style);
        });

        let palette_widget = tool_palette.widget().clone();
        if self.tool_palette.set(tool_palette).is_err() {
            panic!("tool_palette initialized once");
        }

        let content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        content_box.append(&palette_widget);
        canvas.set_hexpand(true);
        content_box.append(&canvas);

        let toolbar_view = libadwaita::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&content_box));

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
