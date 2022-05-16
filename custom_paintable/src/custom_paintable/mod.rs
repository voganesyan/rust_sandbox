mod imp;

use gtk::{gdk, glib};
use opencv::{core::Mat};

glib::wrapper! {
    pub struct CustomPaintable(ObjectSubclass<imp::CustomPaintable>) @implements gdk::Paintable;
}

impl Default for CustomPaintable {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPaintable {
    pub fn new() -> Self {
        glib::Object::new(&[("image", Mat::default())]).expect("Failed to create a CustomPaintable")
    }

    pub fn set_image(&self, image: Mat) {
        println!("set_image");
        self.imp().image = image;
    }
}
