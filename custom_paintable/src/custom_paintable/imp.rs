use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, cairo};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use glib::{BindingFlags, ParamFlags, ParamSpec, ParamSpecObject, Value};


#[derive(Default)]
pub struct CustomPaintable {
    pub image: RefCell<Option<cairo::ImageSurface>>
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for CustomPaintable {
    const NAME: &'static str = "CustomPaintable";
    type Type = super::CustomPaintable;
    type Interfaces = (gdk::Paintable,);
}

impl CustomPaintable {
    pub(super) fn image(&self, _obj: &super::CustomPaintable) -> Option<gtk::Widget> {
        self.image.borrow().clone()
    }

    pub(super) fn set_image(
        &self,
        obj: &super::CustomPaintable,
        widget: Option<&impl IsA<cairo::ImageSurface>>,
    ) {
        let widget = widget.map(|w| w.upcast_ref());
        if widget == self.image.borrow().as_ref() {
            return;
        }

        if let Some(image) = self.image.borrow_mut().take() {
            image.unparent();
        }

        if let Some(w) = widget {
            self.image.replace(Some(w.clone()));
            w.set_parent(obj);
        }

        obj.queue_resize();
        obj.notify("image")
    }


// Trait shared by all GObjects
impl ObjectImpl for CustomPaintable {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![ParamSpecObject::new(
                // Name
                "image",
                // Nickname
                "image",
                // Short description
                "image",
                // Object type
                cairo::ImageSurface::static_type(),
                // The property can be read and written to
                ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "image" => {
                self.set_image(obj, value.get::<cairo::ImageSurface>().ok().as_ref());
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "image" => self.image(obj).to_value(),
            _ => unimplemented!(),
        }
    }

    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        // obj.bind_property("number", obj, "label")
        //     .flags(BindingFlags::SYNC_CREATE)
        //     .build();
    }
}

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
        ctx.paint().unwrap();
    }
}
