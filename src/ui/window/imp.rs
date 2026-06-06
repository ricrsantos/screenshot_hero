use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita::prelude::AdwApplicationWindowExt;
use libadwaita::subclass::application_window::AdwApplicationWindowImpl;
use libadwaita::subclass::window::AdwWindowImpl;

#[derive(Default)]
pub struct MainWindow;

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

        let actions = gio::SimpleActionGroup::new();

        let new_screenshot = gio::SimpleAction::new("new-screenshot", None);
        new_screenshot.connect_activate(|_, _| {});
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

        let placeholder = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let toolbar_view = libadwaita::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&placeholder));

        window.set_content(Some(&toolbar_view));
    }
}

impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
impl AdwWindowImpl for MainWindow {}
impl AdwApplicationWindowImpl for MainWindow {}
