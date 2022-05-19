use gtk::{glib, gdk};
use gtk::prelude::*;
use gtk;

use opencv::core::Vec3b;
use opencv::{prelude::*, videoio, Result};
use std::sync::{Arc, Mutex};


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
}

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

    // Start reading video stream
    let image = Arc::new(Mutex::new(Mat::default()));
    let image = image.clone();
    std::thread::spawn(move || {
        start_reading_frames(image).unwrap();
    });

    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        println!("Tick");
        // we could return gtk::Continue(false) to stop our clock after this tick
        glib::Continue(true)
    };

    // executes the closure once every second
    glib::timeout_add_seconds_local(1, tick);
}
