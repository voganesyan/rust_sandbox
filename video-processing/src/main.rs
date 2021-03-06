use gtk::{cairo, glib, prelude::*};
use ui::UIControls;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencv::core::Vec3b;
use opencv::{core, imgproc, prelude::*, videoio, Result};

mod image_classification;
use image_classification::ImageClassifier;

mod image_processing;
use image_processing::*;

mod ui;

struct ProcessingContext {
    // Output data
    image: Mat,
    class: String,

    // Input preprocessing parameters
    contrast: f64,
    brightness: f64,
    proc_fn: AdjustBrightnessContrastFn,

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

fn get_combo_active_method(combo: &gtk::ComboBoxText) -> AdjustBrightnessContrastFn {
    let func_name = combo.active_text().unwrap();
    let func_name = func_name.as_str();
    ADJUST_BRIGHTNESS_CONTRAST_FN_MAP[func_name]
}

fn calc_scale_factor(image_w: i32, image_h: i32, canvas_w: i32, canvas_h: i32) -> f64 {
    let scale_w = canvas_w as f64 / image_w as f64;
    let scale_h = canvas_h as f64 / image_h as f64;
    scale_w.min(scale_h)
}

fn init_ui(ui: &UIControls) {
    for &func_name in ADJUST_BRIGHTNESS_CONTRAST_FN_MAP.keys() {
        ui.method_combo.append_text(func_name);
    }
    ui.method_combo.set_active(Some(0));
    ui.contrast_scale.set_value(1.0);
    ui.brightness_scale.set_value(0.0);

    ui.model_combo.append_text("MobileNetV3");
    ui.model_combo.set_active(Some(0));
}

fn set_ui_handlers(ui: &UIControls, context: &Arc<Mutex<ProcessingContext>>) {
    let context_clone = context.clone();
    ui.window.connect_close_request(move |_window| {
        context_clone.lock().unwrap().should_stop = true;
        // TODO: join bkgd_thread to avoid segfault when closing the application
        // bkgd_thread.join().unwrap();
        gtk::Inhibit(false)
    });

    let context_clone = context.clone();
    ui.method_combo.connect_changed(move |combo| {
        context_clone.lock().unwrap().proc_fn = get_combo_active_method(combo);
    });

    let context_clone = context.clone();
    ui.contrast_scale.connect_value_changed(move |scale| {
        context_clone.lock().unwrap().contrast = scale.value();
    });

    let context_clone = context.clone();
    ui.brightness_scale.connect_value_changed(move |scale| {
        context_clone.lock().unwrap().brightness = scale.value();
    });

    // Implement drawing function
    let context_clone = context.clone();
    ui.drawing_area.set_draw_func(move |_, cx, width, height| {
        // Clear cairo context
        cx.set_operator(cairo::Operator::Clear);
        cx.set_source_rgba(0., 0., 0., 0.);
        cx.paint().expect("Couldn't fill context");

        // Draw image
        cx.set_operator(cairo::Operator::Source);
        let context = context_clone.lock().unwrap();
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
            cx.set_source_rgb(0.1, 0.1, 0.1);

            // Draw preprocessing label
            cx.move_to(5., height as f64 - 5. - font_size);
            let text = format!(
                "Brightness/Constrast: {:.2} ms",
                context.preprocessing_time.as_micros() as f64 * 1e-3
            );
            cx.show_text(&text).unwrap();

            // Draw classification label
            cx.move_to(5., height as f64 - 5.);
            let text = format!(
                "Image Classification: {:.2} ms; Class: {}",
                context.classification_time.as_micros() as f64 * 1e-3,
                context.class
            );
            cx.show_text(&text).unwrap();
        }
    });
}

fn activate_app(application: &gtk::Application) {
    // Build and init UI
    let ui = ui::build_ui(application);
    init_ui(&ui);

    // Create data for sharing between UI and background threads
    let context = Arc::new(Mutex::new(ProcessingContext {
        image: Mat::default(),
        class: String::from("none"),
        contrast: ui.contrast_scale.value(),
        brightness: ui.brightness_scale.value(),
        proc_fn: get_combo_active_method(&ui.method_combo),
        should_stop: false,
        classification_time: Duration::ZERO,
        preprocessing_time: Duration::ZERO,
    }));

    // Set UI handlers
    set_ui_handlers(&ui, &context);

    // Show window
    ui.window.show();

    // Redraw drawing area every 30 milliseconds
    glib::timeout_add_local(Duration::from_millis(30), move || {
        ui.drawing_area.queue_draw();
        glib::Continue(true)
    });

    // Create classifier
    let classifier = ImageClassifier::new("data/mobilenetv3").unwrap();

    // Open video stream (0 is the default camera)
    let mut capture = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    if !capture.is_opened().unwrap() {
        panic!("Unable to open default camera!");
    }

    // Start background thread with reading video stream and classifying images
    let _bkgd_thread = std::thread::spawn(move || {
        loop {
            // Read frame
            let mut frame = Mat::default();
            capture.read(&mut frame).unwrap();

            // Get context
            let mut context = context.lock().unwrap();

            // Check if it's time to stop
            if context.should_stop {
                break;
            }

            // Process frame
            let mut proc_frame = unsafe { Mat::new_rows_cols(frame.rows(), frame.cols(), frame.typ()).unwrap() };
            let now = std::time::Instant::now();
            (context.proc_fn)(&frame, &mut proc_frame, context.contrast, context.brightness);
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
}

fn main() {
    let app = gtk::Application::new(None, Default::default());
    app.connect_activate(activate_app);
    app.run();
}
