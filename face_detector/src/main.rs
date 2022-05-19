use gtk::{glib, gdk};
use gtk::prelude::*;
use gtk;

fn main() {
    let application =
        gtk::Application::new(None, Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("Face Detector"));
    window.set_default_size(500, 500);

    let paintable = gdk::Paintable::new_empty(300, 300);

    let picture = gtk::Picture::new();
    picture.set_halign(gtk::Align::Center);
    picture.set_size_request(200, 200);
    picture.set_paintable(Some(&paintable));

    window.set_child(Some(&picture));

    window.show();

    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        println!("Tick");
        // we could return gtk::Continue(false) to stop our clock after this tick
        glib::Continue(true)
    };

    // executes the closure once every second
    glib::timeout_add_seconds_local(1, tick);
}
