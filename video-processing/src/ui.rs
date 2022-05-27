use gtk::prelude::*;

pub struct UIControls {
    pub window: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
    pub func_combo: gtk::ComboBoxText,
    pub alpha_scale: gtk::Scale,
    pub beta_scale: gtk::Scale,
    pub model_combo: gtk::ComboBoxText
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
    let func_label = gtk::Label::new(Some("Method"));
    let func_combo = gtk::ComboBoxText::new();

    // Alpha
    let alpha_label = gtk::Label::new(Some("Contrast"));
    let alpha_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 2.0, 0.01);
    alpha_scale.set_draw_value(true);
    alpha_scale.set_value_pos(gtk::PositionType::Left);

    // Beta
    let beta_label = gtk::Label::new(Some("Brightness"));
    let beta_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -100.0, 100.0, 1.0);
    beta_scale.set_draw_value(true);
    beta_scale.set_value_pos(gtk::PositionType::Left);

    // Create grid
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.attach(&func_label, 0, 0, 1, 1);
    grid.attach(&func_combo, 1, 0, 1, 1);
    grid.attach(&alpha_label, 0, 1, 1, 1);
    grid.attach(&alpha_scale, 1, 1, 1, 1);
    grid.attach(&beta_label, 0, 2, 1, 1);
    grid.attach(&beta_scale, 1, 2, 1, 1);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(10);
    grid.set_margin_bottom(10);

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
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(10);
    grid.set_margin_bottom(10);

    imgclass_frame.set_child(Some(&grid));

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal,10);
    hbox.append(&imgproc_frame);
    hbox.append(&imgclass_frame);

    hbox.set_margin_start(10);
    hbox.set_margin_end(10);
    hbox.set_margin_top(10);
    hbox.set_margin_bottom(10);

    vbox.append(&hbox);

    // Create drawing area
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_vexpand(true);
    vbox.append(&drawing_area);

    UIControls {
        window,
        drawing_area,
        func_combo,
        alpha_scale,
        beta_scale,
        model_combo,
    }
}
