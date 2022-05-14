mod custom_paintable;

use custom_paintable::CustomPaintable;
use gtk::prelude::*;
use gtk::glib;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.paintable"),
        Default::default(),
    );
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Custom Paintable"));
    window.set_default_size(500, 500);

    let paintable = CustomPaintable::new();

    let picture = gtk::Picture::new();
    picture.set_halign(gtk::Align::Center);
    picture.set_size_request(200, 200);
    picture.set_paintable(Some(&paintable));

    window.set_child(Some(&picture));
    window.show();

    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        println!("time");

        // let time = current_time();
        // label.set_text(&time);
        paintable.set_image();

        // Now we need to tell all listeners that we've changed out contents
        // so that they can redraw this paintable.
        paintable.invalidate_contents();
        
        // We want this timeout function to be called repeatedly,
        // so we return this value here.
        // If this was a single-shot timeout, we could also
        // return gtk::Continue(false) to stop our clock after this tick
        glib::Continue(true)
    };

    // executes the closure once every second
    glib::timeout_add_seconds_local(1, tick);
}
