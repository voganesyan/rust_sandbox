#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui_extras::RetainedImage;

use std::sync::mpsc;
use std::thread;
use opencv::{
	// highgui,
	prelude::*,
	Result,
	videoio,
};


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
    image: RetainedImage,
    rx: Option<mpsc::Receiver<Mat>>,
}


fn start_sending_frames(tx: mpsc::Sender<Mat>) -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        println!("read");
        tx.send(frame).unwrap();
    }
    Ok(())
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            rx: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Run Video").clicked() {
                let (tx, rx) = mpsc::channel::<Mat>();
                self.rx = Some(rx);
                let _handle = thread::spawn(move || {
                    start_sending_frames(tx).unwrap();
                });
                // handle.join().unwrap();
            }
            ui.heading("This is an image:");
            match &self.rx {
                Some(rx) => {
                    println!("Received Frame");
                    let res = rx.try_recv();
                    match res {
                        Ok(_) => println!("Received Frame"),
                        Err(e) => println!("Could not receive a frame {:?}", e),
                    }
                }
                None => {
                    println!("Receiver is not initialized yet. Click the button.");
                }
            }
            self.image.show(ui);

            ui.heading("This is a rotated image:");
            ui.add(
                egui::Image::new(self.image.texture_id(ctx), self.image.size_vec2())
                    .rotate(45.0_f32.to_radians(), egui::Vec2::splat(0.5)),
            );

            ui.heading("This is an image you can click:");
            ui.add(egui::ImageButton::new(
                self.image.texture_id(ctx),
                self.image.size_vec2(),
            ));
        });
    }
}
