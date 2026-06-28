use gdk_pixbuf::Pixbuf;
use gtk::gdk;
use gtk::prelude::*;

/// Copies `pixbuf` to the system clipboard for `display`.
pub fn copy_to_clipboard(display: &gdk::Display, pixbuf: &Pixbuf) {
    let texture = gdk::Texture::for_pixbuf(pixbuf);
    display.clipboard().set_texture(&texture);
}
