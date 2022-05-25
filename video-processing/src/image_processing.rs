use opencv::core::*;
use rayon::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;



pub type AdjustBrightnessContrastFn = fn(&Mat, f64, f64) -> Mat;

lazy_static!{
    pub static ref ADJUST_BRIGHTNESS_CONTRAST_FN_MAP: HashMap<&'static str, AdjustBrightnessContrastFn> = [
        ("OpenCV", adjust_brightness_contrast_opencv as AdjustBrightnessContrastFn),
        ("Own (Sequential)", adjust_brightness_contrast_own as AdjustBrightnessContrastFn),
        ("Own (Parallel)", adjust_brightness_contrast_own_parallel as AdjustBrightnessContrastFn),
    ].iter().copied().collect();
}

pub fn adjust_brightness_contrast_opencv(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = Mat::default();
    src.convert_to(&mut dst, src.typ(), alpha, beta).unwrap();
    dst
}



fn adjust_value(val: u8, alpha: f64, beta: f64) -> u8 {
    use std::cmp;
    let val = (val as f64 * alpha + beta) as u8;
    cmp::min(cmp::max(val, 0), 255)
}

pub fn adjust_brightness_contrast_own(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = unsafe { Mat::new_rows_cols(src.rows(), src.cols(), src.typ()).unwrap() };
    let height = src.rows() as isize;
    let width = src.cols();
    let src_step = src.mat_step()[0] as isize;
    let dst_step = dst.mat_step()[0] as isize;
    let num_channles = src.mat_step()[1];
    for y in 0..height {
        unsafe {
            let mut src_p = src.data().offset(y * src_step);
            let mut dst_p = dst.data_mut().offset(y * dst_step);
            for _ in 0..width {
                for _ in 0..num_channles {
                    *dst_p = adjust_value(*src_p, alpha, beta);
                    src_p = src_p.offset(1);
                    dst_p = dst_p.offset(1);
                }
            }
        }
    }
    dst
}

pub fn adjust_brightness_contrast_own_parallel(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = unsafe { Mat::new_rows_cols(src.rows(), src.cols(), src.typ()).unwrap() };
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.par_iter().zip(dst_data.par_iter_mut());
    it.for_each(|(src, dst)| *dst = adjust_value(*src, alpha, beta) );
    dst
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
