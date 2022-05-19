use gtk::cairo::{Context, Operator};
use gtk::prelude::{
    BoxExt, DrawingAreaExt, GestureSingleExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::{gtk, send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};
use gtk::glib;
use gtk::cairo;

use std::sync::{Arc, Mutex};

use opencv::core::Vec3b;
use opencv::{prelude::*, core, imgproc, videoio, Result};


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

enum AppMsg {
    AddPoint((f64, f64)),
    ClearPoints,
    UpdateDisplayedImage,
    Resize((i32, i32))
}

struct AppModel {
    width: i32,
    height: i32,
    points: Vec<Point>,
    image: Arc<Mutex<Mat>>
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::AddPoint((x, y)) => {
                self.points.push(Point::new(x, y));
            }
            AppMsg::ClearPoints => {
                self.points.clear();
            }
            AppMsg::Resize((width, height)) => {
                self.width = width;
                self.height = height;
            }
            _ => {}
        }
        true
    }
}

struct Point {
    x: f64,
    y: f64,
    color: Color,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point {
            x,
            y,
            color: Color::random(),
        }
    }
}

struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    fn random() -> Color {
        Color {
            r: rand::random(),
            g: rand::random(),
            b: rand::random(),
        }
    }
}

fn cv_mat_to_cairo_surface(image: &Mat) -> Result<cairo::ImageSurface, cairo::Error> {
    let height = image.rows();
    let width = image.cols();
    let mut surface = gtk::cairo::ImageSurface::create(
        cairo::Format::Rgb24, width, height).unwrap();
    let mut surf_data = surface.data().unwrap();
    // We pass chunks_mut = 4, because cairo::Format::Rgb24 is actually RgbA32 with unused alpha-channel
    for it in image.iter::<Vec3b>().unwrap().zip(surf_data.chunks_mut(4)) {
        let (src, dst) = it;
        // println!("{:?} {:?}", src, dst);
        let src = src.1;
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
    }
    drop(surf_data);
    Ok(surface)
}

fn draw(cx: &Context, model: &AppModel) {
    // Clear context
    cx.set_operator(Operator::Clear);
    cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    cx.paint().expect("Couldn't fill context");

    // Draw image
    cx.set_operator(Operator::Source);
    let image = model.image.lock().unwrap();
    if !image.empty() {
        let size = core::Size::new(model.width, model.height);
        let mut small_image = Mat::default();
        imgproc::resize(&*image, &mut small_image, size, 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();
        let surface = cv_mat_to_cairo_surface(&small_image).unwrap();
        cx.set_source_surface(&surface, 0.0, 0.0).unwrap();
        cx.paint().unwrap();
    }

    // Draw points
    cx.set_operator(Operator::Source);
    for pt in &model.points {
        let c = &pt.color;
        cx.set_source_rgb(c.r, c.g, c.b);
        cx.arc(pt.x, pt.y, 10.0, 0.0, std::f64::consts::PI * 2.0);
        cx.fill().expect("Couldn't fill arc");
    }
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
      main_window = gtk::ApplicationWindow {
        set_default_height: 300,
        set_default_width: 600,
        set_child = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_all: 10,
          set_spacing: 10,
          set_hexpand: true,
          append = &gtk::Label {
            set_label: "Left-click to add circles, resize or right-click to reset!",
          },
          append: area = &gtk::DrawingArea {
            set_vexpand: true,
            set_hexpand: true,
            add_controller = &gtk::GestureClick::new() {
              set_button: 0,
              connect_pressed(sender) => move |controller, _, x, y| {
                if controller.current_button() == gtk::gdk::BUTTON_SECONDARY {
                  send!(sender, AppMsg::ClearPoints);
                } else {
                  send!(sender, AppMsg::AddPoint((x, y)));
                }
              }
            },
            connect_resize(sender) => move |_, x, y| {
              send!(sender, AppMsg::Resize((x, y)))
            }
          },
        }
      }
    }

    additional_fields! {
        handler: relm4::drawing::DrawHandler,
    }

    fn post_init() {
        let mut handler = relm4::drawing::DrawHandler::new().unwrap();
        handler.init(&area);

        // Start reading video stream
        let image = model.image.clone();
        std::thread::spawn(move || {
            start_reading_frames(image).unwrap();
        });

        // Start updating displayed image every second
        glib::timeout_add_seconds_local(1, move || {
            send!(sender, AppMsg::UpdateDisplayedImage);
            glib::Continue(true)
        });
    }

    fn pre_view() {
        let cx = self.handler.get_context().unwrap();
        draw(&cx, &model);
    }
}

fn main() {
    let model = AppModel {
        width: 100,
        height: 100,
        points: Vec::new(),
        image: Arc::new(Mutex::new(Mat::default()))
    };
    let relm = RelmApp::new(model);
    relm.run();
}
