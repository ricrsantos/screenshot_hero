use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;
use libadwaita::prelude::{AdwDialogExt, AlertDialogExt};

pub fn show_error_dialog<W: IsA<gtk::Window>>(parent: &W, title: &str, message: &str) {
    let dialog = build_error_dialog(title, message);
    dialog.present(Some(parent.upcast_ref()));
}

pub fn show_timed_error_dialog<W: IsA<gtk::Window>>(
    parent: &W,
    title: &str,
    message: &str,
    duration: Duration,
) {
    let dialog = build_error_dialog(title, message);
    dialog.present(Some(parent.upcast_ref()));
    wire_timed_error_lifecycle(&dialog, duration, None);
}

pub fn show_timed_error_dialog_standalone<F: FnOnce() + 'static>(
    title: &str,
    message: &str,
    duration: Duration,
    on_done: F,
) {
    let dialog = build_error_dialog(title, message);
    dialog.present(None::<&gtk::Widget>);
    wire_timed_error_lifecycle(&dialog, duration, Some(Box::new(on_done)));
}

fn build_error_dialog(title: &str, message: &str) -> libadwaita::AlertDialog {
    let dialog = libadwaita::AlertDialog::new(Some(title), Some(message));
    dialog.add_response("ok", "OK");
    dialog.set_close_response("ok");
    dialog.set_default_response(Some("ok"));
    dialog
}

fn wire_timed_error_lifecycle(
    dialog: &libadwaita::AlertDialog,
    duration: Duration,
    on_done: Option<Box<dyn FnOnce() + 'static>>,
) {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    let done = Rc::new(Cell::new(false));
    let timer_id = Rc::new(RefCell::new(None::<glib::SourceId>));
    let on_done = Rc::new(RefCell::new(on_done));

    let finish = {
        let done = done.clone();
        let timer_id = timer_id.clone();
        let on_done = on_done.clone();
        move || {
            if done.replace(true) {
                return;
            }

            if let Some(id) = timer_id.borrow_mut().take() {
                id.remove();
            }

            if let Some(callback) = on_done.borrow_mut().take() {
                callback();
            }
        }
    };

    let finish_on_response = finish.clone();
    dialog.connect_response(None, move |_, _| {
        finish_on_response();
    });

    let dialog_for_timeout = dialog.clone();
    let finish_on_timeout = finish;
    let timeout_id = glib::timeout_add_local_once(duration, move || {
        finish_on_timeout();
        let _ = dialog_for_timeout.close();
    });
    *timer_id.borrow_mut() = Some(timeout_id);
}
