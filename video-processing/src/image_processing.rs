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
        ("Own (Parallel Row)", adjust_brightness_contrast_own_parallel_row as AdjustBrightnessContrastFn),

        ].iter().copied().collect();
}

pub fn adjust_brightness_contrast_opencv(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = Mat::default();
    src.convert_to(&mut dst, src.typ(), alpha, beta).unwrap();
    dst
}

#[inline]
fn adjust_value(val: u8, alpha: f64, beta: f64) -> u8 {
    use std::cmp;
    let val = (val as f64 * alpha + beta) as u8;
    cmp::min(cmp::max(val, 0), 255)
}

pub fn adjust_brightness_contrast_own(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = unsafe { Mat::new_rows_cols(src.rows(), src.cols(), src.typ()).unwrap() };
    let lut: Vec<u8> = (0..=255).map(|val| adjust_value(val, alpha, beta)).collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.iter().zip(dst_data.iter_mut());
    it.for_each(|(src, dst)| *dst = lut[*src as usize] );
    dst
}

pub fn adjust_brightness_contrast_own_parallel(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = unsafe { Mat::new_rows_cols(src.rows(), src.cols(), src.typ()).unwrap() };
    let lut: Vec<u8> = (0..=255).map(|val| adjust_value(val, alpha, beta)).collect();

    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.par_iter().zip(dst_data.par_iter_mut());
    it.for_each(|(src, dst)| *dst = lut[*src as usize] );
    dst
}



pub fn adjust_brightness_contrast_own_parallel_row(src: &Mat, alpha: f64, beta: f64) -> Mat {
    let mut dst = unsafe { Mat::new_rows_cols(src.rows(), src.cols(), src.typ()).unwrap() };
    let lut: Vec<u8> = (0..=255).map(|val| adjust_value(val, alpha, beta)).collect();
    
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let chunk_size = (src.cols() * 3) as usize;
    let src_iter = src_data.par_chunks(chunk_size);
    let dst_iter = dst_data.par_chunks_mut(chunk_size);
    let it = src_iter.zip(dst_iter);
    it.for_each(|(src, dst)| {
        let it = src.iter().zip(dst.iter_mut());
        it.for_each(|(src, dst)| {
            *dst = lut[*src as usize];
        });
    });
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
