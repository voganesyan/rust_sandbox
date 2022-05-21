use gtk::{cairo, glib, prelude::*};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencv::core::Vec3b;
use opencv::{core, imgproc, prelude::*, videoio, Result};
mod face_detector;

fn start_reading_frames(shared_frame: Arc<Mutex<Mat>>) -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    if !cam.is_opened()? {
        panic!("Unable to open default camera!");
    }

    // Create detector
    let detector = face_detector::Detector::new("./src/data/mobilenetv3").unwrap();

    loop {
        // Read frame
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        // Detect
        println!("{}", detector.detect(&frame).unwrap());

        // Update shared image
        let mut image = shared_frame.lock().unwrap();
        *image = frame;
    }
}

fn cv_mat_to_cairo_surface(image: &Mat) -> Result<cairo::ImageSurface, cairo::Error> {
    let height = image.rows();
    let width = image.cols();
    let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap();
    let mut surf_data = surface.data().unwrap();
    // We pass chunks_mut = 4, because cairo::Format::Rgb24 is actually RgbA32 with unused alpha-channel
    for it in image.iter::<Vec3b>().unwrap().zip(surf_data.chunks_mut(4)) {
        let (src, dst) = it;
        let src = src.1;
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
    }
    drop(surf_data);
    Ok(surface)
}

fn main() {
    let app = gtk::Application::new(None, Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(application: &gtk::Application) {
    // Start reading video stream
    let image = Arc::new(Mutex::new(Mat::default()));
    let image_clone = image.clone();
    std::thread::spawn(move || {
        start_reading_frames(image_clone).unwrap();
    });

    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Face Detector"));
    window.set_default_size(500, 500);

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_draw_func(move |_, cx, width, height| {
        println!("draw {} {}", width, height);
        // Clear context
        cx.set_operator(cairo::Operator::Clear);
        cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cx.paint().expect("Couldn't fill context");

        // Draw image
        cx.set_operator(cairo::Operator::Source);
        let image = image.lock().unwrap();
        if !image.empty() {
            let size = core::Size::new(width, height);
            let mut small_image = Mat::default();
            imgproc::resize(
                &*image,
                &mut small_image,
                size,
                0.0,
                0.0,
                imgproc::INTER_LINEAR,
            )
            .unwrap();
            let surface = cv_mat_to_cairo_surface(&small_image).unwrap();
            cx.set_source_surface(&surface, 0.0, 0.0).unwrap();
            cx.paint().unwrap();
        }
    });

    window.set_child(Some(&drawing_area));
    window.show();

    glib::timeout_add_local(Duration::from_millis(30), move || {
        drawing_area.queue_draw();
        glib::Continue(true)
    });
}
