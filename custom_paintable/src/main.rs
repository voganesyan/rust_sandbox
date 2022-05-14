mod custom_paintable;

use custom_paintable::CustomPaintable;
use gtk::prelude::*;
use gtk::glib;

use std::sync::{Arc, Mutex};
use std::thread;

use opencv::{imgproc::*, prelude::*, videoio, Result};


fn start_reading_frames(shared_frame: Arc<Mutex<Mat>>) -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        println!("Read Frame: {}", frame.size().unwrap().width);
        let mut image = shared_frame.lock().unwrap();
        *image = frame;
    }
    Ok(())
}


fn main() {
    let shared_frame = Arc::new(Mutex::new(Mat::default()));
    let shared_frame_clone = shared_frame.clone();
    let handle = thread::spawn(move || {
        start_reading_frames(shared_frame_clone).unwrap();
    });
    handle.join().unwrap();

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
