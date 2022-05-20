use std::error::Error;
use std::path::PathBuf;
use std::result::Result;
use tensorflow::Code;
use tensorflow::Graph;
use tensorflow::SavedModelBundle;
use tensorflow::SessionOptions;
use tensorflow::SessionRunArgs;
use tensorflow::Status; 
use tensorflow::Tensor;
use tensorflow::DEFAULT_SERVING_SIGNATURE_DEF_KEY;

use tensorflow::eager::{self, raw_ops, ToTensorHandle};

use opencv::{prelude::*, core::*, imgproc};

pub fn cv_mat_to_tf_tensor(image: &Mat) -> Tensor::<f32> {
    println!("normalized {:?}" , image.size());
    let mut normalized = Mat::default();
    image.convert_to(&mut normalized, CV_32FC3, 1.0 / 1.0, 0.0).unwrap(); 
    // where normalized is an opencv matrix already converted to floating point and normalized to 0..1.
    let mut input_tensor = Tensor::<f32>::new(&[1, 224, 224, 3]);
    let ptr = input_tensor.as_mut_ptr() as *mut std::ffi::c_void;
    let mut input_mat = unsafe {
        Mat::new_rows_cols_with_data(
            224,
            224,
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


pub fn detect(image: Mat) -> Result<(), Box<dyn Error>> {
    let export_dir = "/home/vitaliy/Documents/rust_sandbox/face_detector/src/face_detector/data/mobilenetv3";
    let model_file: PathBuf = [export_dir, "saved_model.pb"].iter().collect();
    if !model_file.exists() {
        return Err(Box::new(
            Status::new_set(
                Code::NotFound,
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
    let opts = eager::ContextOptions::new();
    let ctx = eager::Context::new(opts)?;

    let x = cv_mat_to_tf_tensor(&image);

    // Load the model.
    let mut graph = Graph::new();
    let bundle =
        SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, export_dir)?;
    let session = &bundle.session;

    // get in/out operations
    let signature = bundle
        .meta_graph_def()
        .get_signature(DEFAULT_SERVING_SIGNATURE_DEF_KEY)?;
    let x_info = signature.get_input("input_1")?;
    let op_x = &graph.operation_by_name_required(&x_info.name().name)?;
    let output_info = signature.get_output("Predictions")?;
    let op_output = &graph.operation_by_name_required(&output_info.name().name)?;

    // Run the graph.
    let mut args = SessionRunArgs::new();
    args.add_feed(op_x, 0, &x);
    let token_output = args.request_fetch(op_output, 0);
    session.run(&mut args)?;

    // Check the output.
    let output: Tensor<f32> = args.fetch(token_output)?;

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

    Ok(())
}
