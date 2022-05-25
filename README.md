# rust_sandbox

This is a sandbox repository for experimenting with Rust and some computer vision crates.
The `video-processing` apllication has the following features:
* Reading video stream with [OpenCV](https://github.com/twistedfall/opencv-rust) in the background thread;
* Exposing video stream and some user controls in the UI thread with [Gtk4](https://github.com/gtk-rs/gtk4-rs);
* Brightness/contrast adjustment with different methods:
    * [opencv::convert_to](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html#method.convert_to);
    * own sequential loop-based processing;
    * own parallel [Rayon](https://github.com/rayon-rs/rayon)-based processing;
* Image classification with [TensorFlow](https://github.com/tensorflow/rust) (MobileNetV3);



 
