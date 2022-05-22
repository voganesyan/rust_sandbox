use gtk::{cairo, glib, prelude::*};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencv::core::Vec3b;
use opencv::{core, imgproc, prelude::*, videoio, Result};
mod classifier;

struct ProcessingContext {
    image: Mat,
    class: String
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


fn calc_scale_factor(image_w: i32, image_h: i32, canvas_w: i32, canvas_h: i32) -> f64 {
    let scale_w = canvas_w as f64 / image_w as f64;
    let scale_h = canvas_h as f64 / image_h as f64;
    scale_w.min(scale_h)
}


fn main() {
    let app = gtk::Application::new(None, Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(application: &gtk::Application) {
    // Open video stream (0 is the default camera)
    let mut capture = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    if !capture.is_opened().unwrap() {
        panic!("Unable to open default camera!");
    }

    // Create classifier
    let classifier = classifier::Classifier::new("./src/data/mobilenetv3").unwrap();

    // Create data for sharing between GUI and background threads
    let context = Arc::new(Mutex::new(
        ProcessingContext{
        image: Mat::default(),
        class: String::from("none")
    }));

    // Start background thread with reading video stream and classifying images
    let context_clone = context.clone();
    std::thread::spawn(move || {
        loop {
            // Read frame
            let mut frame = Mat::default();
            capture.read(&mut frame).unwrap();
    
            // Classify frame
            let class = classifier.classify(&frame).unwrap();
    
            // Update shared context
            let mut context = context_clone.lock().unwrap();
            context.image = frame;
            context.class = String::from(class);
        }
    });

    // Create application window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Face Detector"));
    window.set_default_size(500, 500);

    // Create drawing area
    let drawing_area = gtk::DrawingArea::new();
    window.set_child(Some(&drawing_area));

    // Implement drawing function
    drawing_area.set_draw_func(move |_, cx, width, height| {
        // Clear cairo context
        cx.set_operator(cairo::Operator::Clear);
        cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cx.paint().expect("Couldn't fill context");

        // Draw image
        cx.set_operator(cairo::Operator::Source);
        let context = context.lock().unwrap();
        let image = &context.image;
        if !image.empty() {
            let scale_factor = calc_scale_factor(image.cols(), image.rows(), width, height);
            let mut small_image = Mat::default();
            imgproc::resize(
                image,
                &mut small_image,
                core::Size::new(0, 0),
                scale_factor,
                scale_factor,
                imgproc::INTER_LINEAR,
            ).unwrap();
            let surface = cv_mat_to_cairo_surface(&small_image).unwrap();
            let x_shift = (width - small_image.cols()) / 2;
            let y_shift = (height - small_image.rows()) / 2;
            cx.set_source_surface(&surface, x_shift as f64, y_shift as f64).unwrap();
            cx.paint().unwrap();

            // Draw class label
            cx.set_font_size(50.0 * scale_factor);
            cx.set_source_rgb(0.8, 0.1, 0.8);
            cx.move_to(5.0, height as f64 - 5.0);
            cx.show_text(&context.class).unwrap(); 
        }
    });

    // Show window
    window.show();

    // Redraw drawing area every 30 milliseconds
    glib::timeout_add_local(Duration::from_millis(30), move || {
        drawing_area.queue_draw();
        glib::Continue(true)
    });
}
