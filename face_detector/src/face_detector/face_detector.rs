use std::error::Error;
use std::path::PathBuf;
use std::result::Result;
use tensorflow as tf;

use opencv::{prelude::*, core::*, imgproc};

fn cv_mat_to_tf_tensor(image: &Mat) -> tf::Tensor::<f32> {
    let rows = image.rows();
    let cols = image.cols();
    let mut normalized = Mat::default();
    image.convert_to(&mut normalized, CV_32FC3, 1.0, 0.0).unwrap(); 
    // where normalized is an opencv matrix already converted to floating point and normalized to 0..1.
    let mut input_tensor = tf::Tensor::<f32>::new(&[1, rows as u64, cols as u64, 3]);
    let ptr = input_tensor.as_mut_ptr() as *mut std::ffi::c_void;
    let mut input_mat = unsafe {
        Mat::new_rows_cols_with_data(
            rows,
            cols,
            opencv::core::CV_32FC3,
            ptr,
            Mat_AUTO_STEP,
        ).unwrap()
    };
    imgproc::cvt_color(
        &normalized,
        &mut input_mat,
        imgproc::COLOR_BGR2RGB,
        0,
    ).unwrap();
    input_tensor
} 


pub struct Detector {
    op_x: tf::Operation,
    op_output: tf::Operation,
    bundle: tf::SavedModelBundle
}

impl Detector {
    pub fn new() -> Result<Detector, Box<dyn Error>> {
        let export_dir = "/home/vitaliy/Documents/rust_sandbox/face_detector/src/face_detector/data/mobilenetv3";
        let model_file: PathBuf = [export_dir, "saved_model.pb"].iter().collect();
        if !model_file.exists() {
            return Err(Box::new(
                tf::Status::new_set(
                    tf::Code::NotFound,
                    &format!(
                        "Run 'python src/mobilenetv3/create_model.py' to generate \
                         {} and try again.",
                        model_file.display()
                    ),
                )
                .unwrap(),
            ));
        }
    
        // Create an eager execution context
        let opts = tf::eager::ContextOptions::new();
        let _ctx = tf::eager::Context::new(opts)?;
    
        // Load the model.
        let mut graph = tf::Graph::new();

        let bundle =
        tf::SavedModelBundle::load(&tf::SessionOptions::new(), &["serve"], &mut graph, export_dir)?;

        // get in/out operations
        let signature = bundle
            .meta_graph_def()
            .get_signature(tf::DEFAULT_SERVING_SIGNATURE_DEF_KEY)?;
        let x_info = signature.get_input("input_1")?;
        let op_x = graph.operation_by_name_required(&x_info.name().name)?;
        let output_info = signature.get_output("Predictions")?;
        let op_output = graph.operation_by_name_required(&output_info.name().name)?;
        Ok(Detector { op_x, op_output, bundle })
    }

    pub fn detect(&self, image: &Mat) -> Result<usize, Box<dyn Error>> {
        // Scale image
        let size = Size::new(224, 224);
        let mut small_image = Mat::default();
        imgproc::resize(&image, &mut small_image, size, 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();

        let x = cv_mat_to_tf_tensor(&small_image);
    
        // Run the graph.
        let mut args = tf::SessionRunArgs::new();
        args.add_feed(&self.op_x, 0, &x);
        let token_output = args.request_fetch(&self.op_output, 0);
        let session = &self.bundle.session;
        session.run(&mut args).unwrap();
    
        // Check the output.
        let output: tf::Tensor<f32> = args.fetch(token_output)?;
    
        // Calculate argmax of the output
        let (max_idx, _max_val) =
            output
                .iter()
                .enumerate()
                .fold((0, output[0]), |(idx_max, val_max), (idx, val)| {
                    if &val_max > val {
                        (idx_max, val_max)
                    } else {
                        (idx, *val)
                    }
                });
    
        // This index is expected to be identical with that of the Python code,
        // but this is not guaranteed due to floating operations.
        println!("argmax={}", max_idx);
        Ok(max_idx)
    }
}
