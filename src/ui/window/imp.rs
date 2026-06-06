use gtk::glib;
use gtk::subclass::prelude::*;
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

impl ObjectImpl for MainWindow {}
impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
impl AdwWindowImpl for MainWindow {}
impl AdwApplicationWindowImpl for MainWindow {}
