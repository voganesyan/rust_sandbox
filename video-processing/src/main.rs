use gtk::{cairo, glib, prelude::*};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencv::core::Vec3b;
use opencv::{core, imgproc, prelude::*, videoio, Result};

mod image_classification;
use image_classification::ImageClassifier;

mod image_processing;
use image_processing::*;

struct ProcessingContext {
    // Output data
    image: Mat,
    class: String,

    // Input preprocessing parameters
    alpha: f64,
    beta: f64,

    // Input flag to exit processing thread
    should_stop: bool,

    // Output benchmark data
    preprocessing_time: Duration,
    classification_time: Duration,
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
    let classifier = ImageClassifier::new("data/mobilenetv3").unwrap();

    // Create data for sharing between GUI and background threads
    let context = Arc::new(Mutex::new(ProcessingContext {
        image: Mat::default(),
        class: String::from("none"),
        alpha: 1.0,
        beta: 0.0,
        should_stop: false,
        classification_time: Duration::ZERO,
        preprocessing_time: Duration::ZERO, 
    }));

    // Start background thread with reading video stream and classifying images
    let context_clone = context.clone();
    let _bkgd_thread = std::thread::spawn(move || {
        loop {
            // Read frame
            let mut frame = Mat::default();
            capture.read(&mut frame).unwrap();

            // Get context
            let mut context = context_clone.lock().unwrap();
            
            // Check if it's time to stop
            if context.should_stop {
                break;
            }

            // Process frame
            let mut proc_frame = unsafe { Mat::new_rows_cols(frame.rows(), frame.cols(), frame.typ()).unwrap() };
            let now = std::time::Instant::now();
            adjust_brightness_contrast_opencv(&frame, &mut proc_frame, context.alpha, context.beta);
            let proc_duration = now.elapsed();
            
            // Classify frame
            let now = std::time::Instant::now();
            let class = classifier.classify(&proc_frame).unwrap();
            let class_duration = now.elapsed();
            
            // Update context output data
            context.image = proc_frame;
            context.class = String::from(class);
            context.preprocessing_time = proc_duration;
            context.classification_time = class_duration;
        }
    });

    // Create application window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Video Processing"));
    window.set_default_size(500, 500);
    let context_clone = context.clone();
    window.connect_close_request(move |_window| {
        let mut context = context_clone.lock().unwrap();
        context.should_stop = true;
        // TODO: join bkgd_thread to avoid segfault when closing the application
        // bkgd_thread.join().unwrap();
        gtk::Inhibit(false)
    });

    // Create vertical box
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&vbox));
    
    // Create image processing dropdown
    let imgproc_frame = gtk::Frame::new(Some("Image Processing"));

    // Create label
    let func_label = gtk::Label::new(Some("Function"));
    //label.set_vexpand(false);

    // Create dropdown
    let func_combo = gtk::ComboBoxText::new();
    //combo.set_vexpand(false);
    for option in ["OpenCV", "Own (Sequential)", "Own (Parallel)"] {
        func_combo.append_text(option);
    }
    func_combo.set_active(Some(0));

    // Alpha
    let alpha_label = gtk::Label::new(Some("Contrast"));
    let alpha_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 2.0, 0.01);
    alpha_scale.set_value(context.lock().unwrap().alpha);
    alpha_scale.set_draw_value(true);

    // Beta
    let beta_label = gtk::Label::new(Some("Brightness"));
    let beta_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -100.0, 100.0, 1.0);
    beta_scale.set_value(context.lock().unwrap().beta);
    beta_scale.set_draw_value(true);

    let context_clone = context.clone();
    alpha_scale.connect_value_changed(move |scale| {
        let mut context = context_clone.lock().unwrap();
        context.alpha = scale.value();
    });

    let context_clone = context.clone();
    beta_scale.connect_value_changed(move |scale| {
        let mut context = context_clone.lock().unwrap();
        context.beta = scale.value();
    });

    // Create grid
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.attach(&func_label, 0, 0, 1, 1);
    grid.attach(&func_combo, 1, 0, 1, 1);
    grid.attach(&alpha_label, 0, 1, 1, 1);
    grid.attach(&alpha_scale, 1, 1, 1, 1);
    grid.attach(&beta_label, 0, 2, 1, 1);
    grid.attach(&beta_scale, 1, 2, 1, 1);

    imgproc_frame.set_child(Some(&grid));
    vbox.append(&imgproc_frame);

    // Create drawing area
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_vexpand(true);
    vbox.append(&drawing_area);

    // Implement drawing function
    drawing_area.set_draw_func(move |_, cx, width, height| {
        // Clear cairo context
        cx.set_operator(cairo::Operator::Clear);
        cx.set_source_rgba(0., 0., 0., 0.);
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
            )
            .unwrap();
            let surface = cv_mat_to_cairo_surface(&small_image).unwrap();
            let x_shift = (width - small_image.cols()) / 2;
            let y_shift = (height - small_image.rows()) / 2;
            cx.set_source_surface(&surface, x_shift as f64, y_shift as f64)
                .unwrap();
            cx.paint().unwrap();

            // Draw text
            let font_size = 50. * scale_factor;
            cx.set_font_size(font_size);
            cx.set_source_rgb(0.8, 0.1, 0.8);
            
            // Draw preprocessing label
            cx.move_to(5., height as f64 - 5. - font_size);
            let text = format!("Preprocessing:  {:.2} ms",
                context.preprocessing_time.as_micros() as f64 * 1e-3);
            cx.show_text(&text).unwrap();

            // Draw classification label
            cx.move_to(5., height as f64 - 5.);
            let text = format!("Classification: {:.2} ms; Class: {}",
             context.classification_time.as_micros() as f64 * 1e-3,
             context.class);
            cx.show_text(&text).unwrap();
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
