#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::ColorImage;
use egui_extras::RetainedImage;
use opencv::core::CV_8UC3;

use opencv::{imgproc::*, prelude::*, videoio, Result};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    image: Arc<Mutex<Mat>>,
}

fn start_sending_frames(shared_frame: Arc<Mutex<Mat>>) -> Result<()> {
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

impl Default for MyApp {
    fn default() -> Self {
        let image = Mat::zeros(100, 100, CV_8UC3).unwrap().to_mat().unwrap();
        Self {
            image: Arc::new(Mutex::new(image)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Run Video").clicked() {
                let image = self.image.clone();
                let _handle = thread::spawn(move || {
                    start_sending_frames(image).unwrap();
                });
                // handle.join().unwrap();
            }
            ui.heading("This is an image:");

            let frame_guard = self.image.lock().unwrap();
            let frame = &*frame_guard;
            let size = [frame.cols() as _, frame.rows() as _];
            let mut frame_rgba = Mat::default();
            cvt_color(frame, &mut frame_rgba, COLOR_BGR2RGBA, 0);
            let frame_data = frame_rgba.data_bytes().unwrap();
            let color_image = ColorImage::from_rgba_unmultiplied(size, frame_data);
            let image = RetainedImage::from_color_image("opencv-frame", color_image);
            image.show(ui);

            // Tell the backend to repaint as soon as possible
            ctx.request_repaint();
        });
    }
}
