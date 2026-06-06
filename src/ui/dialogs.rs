use gtk::prelude::*;
use libadwaita::prelude::AdwDialogExt;

pub fn show_error_dialog<W: IsA<gtk::Window>>(parent: &W, title: &str, message: &str) {
    let dialog = libadwaita::AlertDialog::new(Some(title), Some(message));
    dialog.present(Some(parent.upcast_ref()));
}
