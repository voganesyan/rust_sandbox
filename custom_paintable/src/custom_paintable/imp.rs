use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, cairo};

use opencv::{prelude::*};

#[derive(Default)]
pub struct CustomPaintable {
    pub image: Mat,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for CustomPaintable {
    const NAME: &'static str = "CustomPaintable";
    type Type = super::CustomPaintable;
    type Interfaces = (gdk::Paintable,);
}

// Trait shared by all GObjects
impl ObjectImpl for CustomPaintable {}

// Trait shared by all paintables
impl PaintableImpl for CustomPaintable {
    fn flags(&self, _paintable: &Self::Type) -> gdk::PaintableFlags {
        // Fixed size
        gdk::PaintableFlags::SIZE
    }

    fn intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
        200
    }

    fn intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
        200
    }

    fn snapshot(&self, _paintable: &Self::Type, snapshot: &gdk::Snapshot, width: f64, height: f64) {
        println!("snapshot");
        let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();
        // Draw image
        let ctx = snapshot.append_cairo(&graphene::Rect::new(0_f32, 0_f32, width as f32, height as f32));
        ctx.set_source_rgb(1.0, 0.0, 0.0);

        let rows = self.image.rows();
        let cols = self.image.cols();
        let stride = self.image.mat_step()[0];

        let n_bytes = rows as usize * stride;
        let data = unsafe { std::slice::from_raw_parts(self.image.data(), n_bytes) };
        let data = data.to_vec();
        let image_surface = cairo::ImageSurface::create_for_data(
            data,
            cairo::Format::Rgb24,
            cols,
            rows,
            stride as i32
        );
        ctx.paint().unwrap();
        // // Draw a linear gradient
        // snapshot.append_linear_gradient(
        //     &graphene::Rect::new(0_f32, 0_f32, width as f32, height as f32),
        //     &graphene::Point::new(0f32, 0f32),
        //     &graphene::Point::new(width as f32, height as f32),
        //     &[
        //         gsk::ColorStop::new(0.0, gdk::RGBA::RED),
        //         gsk::ColorStop::new(0.15, gdk::RGBA::new(1.0, 127_f32 / 255_f32, 0.0, 1.0)),
        //         gsk::ColorStop::new(0.3, gdk::RGBA::new(1.0, 1.0, 0.0, 1.0)),
        //         gsk::ColorStop::new(0.45, gdk::RGBA::GREEN),
        //         gsk::ColorStop::new(0.6, gdk::RGBA::BLUE),
        //         gsk::ColorStop::new(
        //             0.75,
        //             gdk::RGBA::new(75_f32 / 255_f32, 0.0, 130_f32 / 255_f32, 1.0),
        //         ),
        //         gsk::ColorStop::new(0.9, gdk::RGBA::new(143_f32 / 255_f32, 0.0, 1.0, 1.0)),
        //     ],
        // );
    }
}
