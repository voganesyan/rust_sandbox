use gtk::prelude::*;

pub struct UIControls {
    pub window: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
    pub method_combo: gtk::ComboBoxText,
    pub contrast_scale: gtk::Scale,
    pub brightness_scale: gtk::Scale,
    pub model_combo: gtk::ComboBoxText,
}

const BOX_MARGIN: i32 = 10;

trait SetMargin: WidgetExt {
    fn set_margin(&self, margin: i32);
}

impl<T> SetMargin for T where T: WidgetExt {
    fn set_margin(&self, margin: i32) {
        self.set_margin_start(margin);
        self.set_margin_end(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }
}

pub fn build_ui(application: &gtk::Application) -> UIControls {
    // Create application window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Video Processing"));
    window.set_default_size(500, 500);

    // Create vertical box
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&vbox));

    // Create image processing controls
    let imgproc_frame = gtk::Frame::new(Some("Brightness/Contrast"));

    // Method
    let method_label = gtk::Label::new(Some("Method"));
    let method_combo = gtk::ComboBoxText::new();

    // Contrast
    let contrast_label = gtk::Label::new(Some("Contrast"));
    let contrast_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 2.0, 0.01);
    contrast_scale.set_draw_value(true);
    contrast_scale.set_value_pos(gtk::PositionType::Left);

    // Brightness
    let brightness_label = gtk::Label::new(Some("Brightness"));
    let brightness_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -100.0, 100.0, 1.0);
    brightness_scale.set_draw_value(true);
    brightness_scale.set_value_pos(gtk::PositionType::Left);

    // Create grid
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.attach(&method_label, 0, 0, 1, 1);
    grid.attach(&method_combo, 1, 0, 1, 1);
    grid.attach(&contrast_label, 0, 1, 1, 1);
    grid.attach(&contrast_scale, 1, 1, 1, 1);
    grid.attach(&brightness_label, 0, 2, 1, 1);
    grid.attach(&brightness_scale, 1, 2, 1, 1);
    grid.set_margin(BOX_MARGIN);

    imgproc_frame.set_child(Some(&grid));

    // Create image classification controls
    let imgclass_frame = gtk::Frame::new(Some("Image Classification"));

    // Model
    let model_label = gtk::Label::new(Some("Model"));
    let model_combo = gtk::ComboBoxText::new();

    // Create grid
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.attach(&model_label, 0, 0, 1, 1);
    grid.attach(&model_combo, 1, 0, 1, 1);
    grid.set_margin(BOX_MARGIN);

    imgclass_frame.set_child(Some(&grid));

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, BOX_MARGIN);
    hbox.append(&imgproc_frame);
    hbox.append(&imgclass_frame);
    hbox.set_margin(BOX_MARGIN);

    vbox.append(&hbox);

    // Create drawing area
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_vexpand(true);
    vbox.append(&drawing_area);

    UIControls {
        window,
        drawing_area,
        method_combo,
        contrast_scale,
        brightness_scale,
        model_combo,
    }
}