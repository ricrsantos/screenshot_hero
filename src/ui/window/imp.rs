use std::cell::{OnceCell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
use crate::export::{self, auto_export, exporter, renderer};
use crate::persistence::{
    is_portal_document_path, PersistenceError, ProjectManager, ProjectMetadata, SheroProject,
    SourceImageRecord, ViewState,
};
use crate::settings::AppSettings;
use crate::ui::preferences::PreferencesWindow;
use crate::ui::ToolPalette;

pub struct MainWindow {
    pub(crate) canvas: OnceCell<Canvas>,
    tool_palette: OnceCell<ToolPalette>,
    zoom_label: OnceCell<gtk::Label>,
    project_manager: RefCell<ProjectManager>,
    clipboard_debounce: RefCell<Option<glib::SourceId>>,
    settings: OnceCell<gio::Settings>,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            canvas: OnceCell::new(),
            tool_palette: OnceCell::new(),
            zoom_label: OnceCell::new(),
            project_manager: RefCell::new(ProjectManager::new()),
            clipboard_debounce: RefCell::new(None),
            settings: OnceCell::new(),
        }
    }
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

        match AppSettings::try_new() {
            Some(app_settings) => {
                let _ = self.settings.set(app_settings.settings().clone());
            }
            None => log::warn!(
                "GSettings schema unavailable; using hardcoded defaults for automation settings"
            ),
        }

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

        let save_project = gio::SimpleAction::new("save-project", None);
        save_project.set_enabled(false);

        let export_png = gio::SimpleAction::new("export-png", None);
        export_png.set_enabled(false);
        let export_jpeg = gio::SimpleAction::new("export-jpeg", None);
        export_jpeg.set_enabled(false);
        let copy_to_clipboard_action = gio::SimpleAction::new("copy-to-clipboard", None);
        copy_to_clipboard_action.set_enabled(false);

        let new_screenshot = gio::SimpleAction::new("new-screenshot", None);
        let window_for_capture = window.clone();
        let canvas_for_capture = canvas.clone();
        let save_project_for_capture = save_project.clone();
        let export_png_for_capture = export_png.clone();
        let export_jpeg_for_capture = export_jpeg.clone();
        let copy_for_capture = copy_to_clipboard_action.clone();
        new_screenshot.connect_activate(move |_, _| {
            let window = window_for_capture.clone();
            let canvas = canvas_for_capture.clone();
            let save_project = save_project_for_capture.clone();
            let export_png = export_png_for_capture.clone();
            let export_jpeg = export_jpeg_for_capture.clone();
            let copy_to_clipboard_action = copy_for_capture.clone();
            glib::spawn_future_local(async move {
                match CaptureService::capture().await {
                    Ok(Some(image)) => {
                        canvas.set_image(image);
                        update_image_dependent_actions(
                            &canvas,
                            &save_project,
                            &export_png,
                            &export_jpeg,
                            &copy_to_clipboard_action,
                        );
                    }
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
        let save_project_for_open = save_project.clone();
        let export_png_for_open = export_png.clone();
        let export_jpeg_for_open = export_jpeg.clone();
        let copy_for_open = copy_to_clipboard_action.clone();
        open_file.connect_activate(move |_, _| {
            let window = window_for_open.clone();
            let canvas = canvas_for_open.clone();
            let save_project = save_project_for_open.clone();
            let export_png = export_png_for_open.clone();
            let export_jpeg = export_jpeg_for_open.clone();
            let copy_to_clipboard_action = copy_for_open.clone();
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
                    Ok(image) => {
                        canvas.set_image(image);
                        update_image_dependent_actions(
                            &canvas,
                            &save_project,
                            &export_png,
                            &export_jpeg,
                            &copy_to_clipboard_action,
                        );
                    }
                    Err(error) => {
                        let message = format_load_error(&path, &error);
                        log::error!("Failed to load image: {message}");
                        show_error_dialog(&window, "Open Failed", &message);
                    }
                }
            });
        });
        actions.add_action(&open_file);

        let window_for_save = window.clone();
        let canvas_for_save = canvas.clone();
        let save_project_for_save = save_project.clone();
        save_project.connect_activate(move |_, _| {
            let window = window_for_save.clone();
            let canvas = canvas_for_save.clone();
            let save_project = save_project_for_save.clone();
            glib::spawn_future_local(async move {
                let Some(snapshot) = build_project_snapshot(&canvas) else {
                    return;
                };

                let current_path = window.imp().project_manager.borrow().current_path.clone();
                let path = if let Some(path) = current_path {
                    path
                } else {
                    let Some(path) = show_save_project_dialog(&window).await else {
                        return;
                    };
                    path
                };

                match window
                    .imp()
                    .project_manager
                    .borrow_mut()
                    .save(&path, snapshot)
                {
                    Ok(()) => {
                        update_window_title(&window, &path);
                        save_project.set_enabled(canvas.source_image_path().is_some());
                    }
                    Err(err) => {
                        let message = format_persistence_error(&err);
                        log::error!("Save failed: {message}");
                        show_error_dialog(&window, "Save Failed", &message);
                    }
                }
            });
        });
        actions.add_action(&save_project);

        let save_project_as = gio::SimpleAction::new("save-project-as", None);
        let window_for_save_as = window.clone();
        let canvas_for_save_as = canvas.clone();
        let save_project_for_save_as = save_project.clone();
        save_project_as.connect_activate(move |_, _| {
            let window = window_for_save_as.clone();
            let canvas = canvas_for_save_as.clone();
            let save_project = save_project_for_save_as.clone();
            glib::spawn_future_local(async move {
                let Some(snapshot) = build_project_snapshot(&canvas) else {
                    return;
                };

                let Some(path) = show_save_project_dialog(&window).await else {
                    return;
                };

                match window
                    .imp()
                    .project_manager
                    .borrow_mut()
                    .save_as(&path, snapshot)
                {
                    Ok(()) => {
                        update_window_title(&window, &path);
                        save_project.set_enabled(canvas.source_image_path().is_some());
                    }
                    Err(err) => {
                        let message = format_persistence_error(&err);
                        log::error!("Save As failed: {message}");
                        show_error_dialog(&window, "Save Failed", &message);
                    }
                }
            });
        });
        actions.add_action(&save_project_as);

        let open_project = gio::SimpleAction::new("open-project", None);
        let window_for_open_project = window.clone();
        let canvas_for_open_project = canvas.clone();
        let save_project_for_open_project = save_project.clone();
        let export_png_for_open_project = export_png.clone();
        let export_jpeg_for_open_project = export_jpeg.clone();
        let copy_for_open_project = copy_to_clipboard_action.clone();
        open_project.connect_activate(move |_, _| {
            let window = window_for_open_project.clone();
            let canvas = canvas_for_open_project.clone();
            let save_project = save_project_for_open_project.clone();
            let export_png = export_png_for_open_project.clone();
            let export_jpeg = export_jpeg_for_open_project.clone();
            let copy_to_clipboard_action = copy_for_open_project.clone();
            glib::spawn_future_local(async move {
                let Some(path) = show_open_project_dialog(&window).await else {
                    return;
                };

                let project = match window.imp().project_manager.borrow_mut().open(&path) {
                    Ok(project) => project,
                    Err(err) => {
                        let message = format_persistence_error(&err);
                        log::error!("Open project failed: {message}");
                        show_error_dialog(&window, "Open Failed", &message);
                        return;
                    }
                };

                if !Path::new(&project.source_image.path).exists() {
                    let message = format!("Source image not found: {}", project.source_image.path);
                    log::error!("{message}");
                    show_error_dialog(&window, "Open Failed", &message);
                    return;
                }

                let image_path = PathBuf::from(&project.source_image.path);
                let image = match FileLoader::load_from_path(&image_path) {
                    Ok(image) => image,
                    Err(error) => {
                        let message = format_load_error(&image_path, &error);
                        log::error!("Failed to load source image: {message}");
                        show_error_dialog(&window, "Open Failed", &message);
                        return;
                    }
                };

                canvas.set_image(image);
                canvas.restore_annotations(project.annotations);
                canvas.restore_zoom_pan(
                    project.view_state.zoom,
                    project.view_state.pan_x,
                    project.view_state.pan_y,
                );
                canvas.imp().history.borrow_mut().clear();
                update_window_title(&window, &path);
                update_image_dependent_actions(
                    &canvas,
                    &save_project,
                    &export_png,
                    &export_jpeg,
                    &copy_to_clipboard_action,
                );
            });
        });
        actions.add_action(&open_project);

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

        let window_for_export_png = window.clone();
        let canvas_for_export_png = canvas.clone();
        export_png.connect_activate(move |_, _| {
            let window = window_for_export_png.clone();
            let canvas = canvas_for_export_png.clone();
            glib::spawn_future_local(async move {
                let Some(path) = show_export_png_dialog(&window).await else {
                    return;
                };
                export_rendered_image(&window, &canvas, &path, ExportFormat::Png).await;
            });
        });
        actions.add_action(&export_png);

        let window_for_export_jpeg = window.clone();
        let canvas_for_export_jpeg = canvas.clone();
        export_jpeg.connect_activate(move |_, _| {
            let window = window_for_export_jpeg.clone();
            let canvas = canvas_for_export_jpeg.clone();
            glib::spawn_future_local(async move {
                let Some(path) = show_export_jpeg_dialog(&window).await else {
                    return;
                };
                export_rendered_image(&window, &canvas, &path, ExportFormat::Jpeg).await;
            });
        });
        actions.add_action(&export_jpeg);

        let canvas_for_copy = canvas.clone();
        copy_to_clipboard_action.connect_activate(move |_, _| {
            copy_canvas_to_clipboard(&canvas_for_copy);
        });
        actions.add_action(&copy_to_clipboard_action);

        let show_preferences = gio::SimpleAction::new("show-preferences", None);
        let window_for_prefs = window.clone();
        show_preferences.connect_activate(move |_, _| {
            let imp = window_for_prefs.imp();
            if let Some(settings) = imp.settings.get() {
                let prefs = PreferencesWindow::new(settings);
                prefs.set_transient_for(Some(&window_for_prefs));
                prefs.present();
            } else {
                log::warn!("Cannot open preferences: GSettings schema unavailable");
            }
        });
        actions.add_action(&show_preferences);

        let undo_for_cb = undo.clone();
        let redo_for_cb = redo.clone();
        let canvas_for_annotation_cb = canvas.clone();
        let window_for_annotation_cb = window.clone();
        canvas.on_annotation_changed(move || {
            undo_for_cb.set_enabled(canvas_for_annotation_cb.can_undo());
            redo_for_cb.set_enabled(canvas_for_annotation_cb.can_redo());

            let imp = window_for_annotation_cb.imp();

            if settings_boolean(&imp.settings, "auto-save-enabled", true) {
                if let Some(snapshot) = build_project_snapshot(&canvas_for_annotation_cb) {
                    imp.project_manager.borrow().maybe_auto_save(snapshot);
                }
            }

            if settings_boolean(&imp.settings, "auto-clipboard-enabled", true) {
                cancel_clipboard_debounce(&imp.clipboard_debounce);

                let canvas = canvas_for_annotation_cb.clone();
                let window = window_for_annotation_cb.clone();
                let id = glib::timeout_add_local_once(Duration::from_millis(300), move || {
                    copy_canvas_to_clipboard(&canvas);
                    *window.imp().clipboard_debounce.borrow_mut() = None;
                });
                *imp.clipboard_debounce.borrow_mut() = Some(id);
            }

            if settings_boolean(&imp.settings, "auto-export-enabled", false) {
                if let Some(source_path) = canvas_for_annotation_cb.source_image_path() {
                    if let Some(source) = canvas_for_annotation_cb.source_pixbuf() {
                        let annotations = canvas_for_annotation_cb.all_annotations();
                        if let Some(pixbuf) = renderer::render_to_pixbuf(&source, &annotations) {
                            let suffix =
                                settings_string(&imp.settings, "auto-export-suffix", "_shero");
                            let export_path =
                                auto_export::build_auto_export_path(&source_path, &suffix);
                            if let Err(err) = exporter::export_png(&pixbuf, &export_path) {
                                log::warn!("Auto-export failed: {err}");
                            }
                        }
                    }
                }
            }
        });

        window.insert_action_group("win", Some(&actions));

        let header = libadwaita::HeaderBar::new();

        let new_button = gtk::Button::builder()
            .label("New Screenshot")
            .action_name("win.new-screenshot")
            .build();
        header.pack_start(&new_button);

        let copy_button = gtk::Button::builder()
            .label("Copy")
            .action_name("win.copy-to-clipboard")
            .build();
        header.pack_start(&copy_button);

        let file_section = gio::Menu::new();
        file_section.append(Some("Open File"), Some("win.open-file"));
        file_section.append(Some("Open Project"), Some("win.open-project"));
        file_section.append(Some("Save"), Some("win.save-project"));

        let export_section = gio::Menu::new();
        export_section.append(Some("Export PNG"), Some("win.export-png"));
        export_section.append(Some("Export JPEG"), Some("win.export-jpeg"));

        let app_section = gio::Menu::new();
        app_section.append(Some("Preferences"), Some("win.show-preferences"));

        let file_menu = gio::Menu::new();
        file_menu.append_section(None, &file_section);
        file_menu.append_section(Some("Export"), &export_section);
        file_menu.append_section(None, &app_section);

        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&file_menu)
            .build();

        header.pack_end(&menu_button);

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

        // Zoom controls — floating pill overlay at the bottom-right of the canvas
        let zoom_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        zoom_bar.add_css_class("osd");
        zoom_bar.add_css_class("linked");
        zoom_bar.set_halign(gtk::Align::End);
        zoom_bar.set_valign(gtk::Align::End);
        zoom_bar.set_margin_end(12);
        zoom_bar.set_margin_bottom(12);

        let zoom_out_btn = gtk::Button::from_icon_name("zoom-out-symbolic");
        zoom_out_btn.set_action_name(Some("win.zoom-out"));
        zoom_out_btn.set_tooltip_text(Some("Zoom Out"));
        zoom_bar.append(&zoom_out_btn);

        zoom_label.set_margin_start(10);
        zoom_label.set_margin_end(10);
        zoom_bar.append(&zoom_label);

        let zoom_in_btn = gtk::Button::from_icon_name("zoom-in-symbolic");
        zoom_in_btn.set_action_name(Some("win.zoom-in"));
        zoom_in_btn.set_tooltip_text(Some("Zoom In"));
        zoom_bar.append(&zoom_in_btn);

        let zoom_100_btn = gtk::Button::from_icon_name("zoom-original-symbolic");
        zoom_100_btn.set_action_name(Some("win.zoom-100"));
        zoom_100_btn.set_tooltip_text(Some("Zoom to Actual Size (1:1)"));
        zoom_bar.append(&zoom_100_btn);

        let zoom_fit_btn = gtk::Button::from_icon_name("zoom-fit-best-symbolic");
        zoom_fit_btn.set_action_name(Some("win.zoom-fit"));
        zoom_fit_btn.set_tooltip_text(Some("Zoom to Fit"));
        zoom_bar.append(&zoom_fit_btn);

        let canvas_overlay = gtk::Overlay::new();
        canvas_overlay.set_hexpand(true);
        canvas_overlay.set_child(Some(&canvas));
        canvas_overlay.add_overlay(&zoom_bar);
        content_box.append(&canvas_overlay);

        let toolbar_view = libadwaita::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&content_box));

        window.set_content(Some(&toolbar_view));
    }
}

fn build_project_snapshot(canvas: &Canvas) -> Option<SheroProject> {
    let path = canvas.source_image_path()?;
    let (width, height) = canvas.source_image_dimensions()?;
    let (pan_x, pan_y) = canvas.pan_offset();

    Some(SheroProject {
        version: 1,
        source_image: SourceImageRecord {
            path: path.to_string_lossy().into_owned(),
            width,
            height,
        },
        annotations: canvas.all_annotations(),
        view_state: ViewState {
            zoom: canvas.zoom_level(),
            pan_x,
            pan_y,
        },
        metadata: ProjectMetadata {
            created_at: String::new(),
            modified_at: String::new(),
            app_version: String::new(),
        },
    })
}

fn settings_boolean(settings: &OnceCell<gio::Settings>, key: &str, default: bool) -> bool {
    settings.get().map(|s| s.boolean(key)).unwrap_or(default)
}

fn settings_string(settings: &OnceCell<gio::Settings>, key: &str, default: &str) -> String {
    settings
        .get()
        .map(|s| s.string(key).to_string())
        .unwrap_or_else(|| default.to_string())
}

fn cancel_clipboard_debounce(debounce: &RefCell<Option<glib::SourceId>>) {
    if let Some(id) = debounce.borrow_mut().take() {
        // g_source_remove returns FALSE if the source already fired; SourceId::remove panics.
        unsafe {
            glib::ffi::g_source_remove(id.as_raw());
        }
    }
}

fn update_image_dependent_actions(
    canvas: &Canvas,
    save_project: &gio::SimpleAction,
    export_png: &gio::SimpleAction,
    export_jpeg: &gio::SimpleAction,
    copy_to_clipboard: &gio::SimpleAction,
) {
    let enabled = canvas.source_image_path().is_some();
    save_project.set_enabled(enabled);
    export_png.set_enabled(enabled);
    export_jpeg.set_enabled(enabled);
    copy_to_clipboard.set_enabled(enabled);
}

enum ExportFormat {
    Png,
    Jpeg,
}

async fn export_rendered_image(
    window: &super::MainWindow,
    canvas: &Canvas,
    path: &Path,
    format: ExportFormat,
) {
    let Some(source) = canvas.source_pixbuf() else {
        log::error!("Export failed: no source image loaded");
        return;
    };
    let annotations = canvas.all_annotations();
    let Some(pixbuf) = renderer::render_to_pixbuf(&source, &annotations) else {
        log::error!("Export failed: could not render image");
        return;
    };

    let result = match format {
        ExportFormat::Png => exporter::export_png(&pixbuf, path),
        ExportFormat::Jpeg => exporter::export_jpeg(&pixbuf, path),
    };

    if let Err(err) = result {
        log::error!("Export failed: {err}");
        show_error_dialog(window, "Export Failed", &err.to_string());
    }
}

fn copy_canvas_to_clipboard(canvas: &Canvas) {
    let Some(source) = canvas.source_pixbuf() else {
        log::error!("Clipboard copy failed: no source image loaded");
        return;
    };
    let annotations = canvas.all_annotations();
    let Some(pixbuf) = renderer::render_to_pixbuf(&source, &annotations) else {
        log::error!("Clipboard copy failed: could not render image");
        return;
    };
    let Some(display) = gtk::gdk::Display::default() else {
        log::error!("Clipboard copy failed: no display available");
        return;
    };
    export::copy_to_clipboard(&display, &pixbuf);
}

async fn show_export_png_dialog(window: &super::MainWindow) -> Option<PathBuf> {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("PNG Images"));
    filter.add_mime_type("image/png");
    filter.add_pattern("*.png");

    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::new();
    dialog.set_title("Export PNG");
    dialog.set_modal(true);
    dialog.set_initial_name(Some("export.png"));
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));

    match dialog.save_future(Some(window)).await {
        Ok(file) => file.path(),
        Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => None,
        Err(error) => {
            log::error!("Export PNG dialog failed: {error}");
            None
        }
    }
}

async fn show_export_jpeg_dialog(window: &super::MainWindow) -> Option<PathBuf> {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("JPEG Images"));
    filter.add_mime_type("image/jpeg");
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.jpeg");

    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::new();
    dialog.set_title("Export JPEG");
    dialog.set_modal(true);
    dialog.set_initial_name(Some("export.jpg"));
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));

    match dialog.save_future(Some(window)).await {
        Ok(file) => file.path(),
        Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => None,
        Err(error) => {
            log::error!("Export JPEG dialog failed: {error}");
            None
        }
    }
}

fn update_window_title(window: &super::MainWindow, path: &Path) {
    window.set_title(Some(&format!(
        "{} - Screenshot Hero",
        project_display_name(path)
    )));
}

/// Portal staging paths look like `.xdp-cap2.shero.tmp-ifkkL1`; extract the user-chosen name.
fn project_display_name(path: &Path) -> String {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return path.display().to_string();
    };

    if let Some(rest) = name.strip_prefix(".xdp-") {
        if let Some(chosen) = rest.split(".tmp-").next() {
            return chosen.to_string();
        }
    }

    name.to_string()
}

fn format_persistence_error(err: &PersistenceError) -> String {
    err.to_string()
}

fn shero_file_filter() -> gtk::FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Screenshot Hero Projects"));
    filter.add_pattern("*.shero");
    filter
}

async fn show_save_project_dialog(window: &super::MainWindow) -> Option<PathBuf> {
    let filter = shero_file_filter();
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::new();
    dialog.set_title("Save Project");
    dialog.set_modal(true);
    dialog.set_initial_name(Some("project.shero"));
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));

    let file = match dialog.save_future(Some(window)).await {
        Ok(file) => file,
        Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => return None,
        Err(error) => {
            log::error!("Save dialog failed: {error}");
            return None;
        }
    };

    let Some(mut path) = file.path() else {
        return None;
    };

    // Flatpak grants write access only to the exact portal staging path (e.g.
    // `.xdp-cap2.shero.tmp-ifkkL1`). Mutating the extension would point at an
    // unauthorized neighbor path and leave an empty staging file behind.
    if !is_portal_document_path(&path) && path.extension().is_none_or(|ext| ext != "shero") {
        path.set_extension("shero");
    }

    Some(path)
}

async fn show_open_project_dialog(window: &super::MainWindow) -> Option<PathBuf> {
    let filter = shero_file_filter();
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::new();
    dialog.set_title("Open Project");
    dialog.set_modal(true);
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));

    match dialog.open_future(Some(window)).await {
        Ok(file) => file.path(),
        Err(error) if error.matches(gio::IOErrorEnum::Cancelled) => None,
        Err(error) => {
            log::error!("Open project dialog failed: {error}");
            None
        }
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
