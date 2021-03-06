use lazy_static::lazy_static;
use opencv::core::*;
use rayon::prelude::*;
use std::collections::HashMap;

pub type AdjustBrightnessContrastFn = fn(&Mat, &mut Mat, f64, f64);

lazy_static! {
    pub static ref ADJUST_BRIGHTNESS_CONTRAST_FN_MAP: HashMap<&'static str, AdjustBrightnessContrastFn> =
        [
            (
                "OpenCV::convertTo",
                adjust_brightness_contrast_opencv as AdjustBrightnessContrastFn
            ),
            (
                "Own (Sequential)",
                adjust_brightness_contrast_own as AdjustBrightnessContrastFn
            ),
            (
                "Own (Parallel)",
                adjust_brightness_contrast_own_parallel as AdjustBrightnessContrastFn
            ),
            (
                "Own (Parallel Rows)",
                adjust_brightness_contrast_own_parallel_rows as AdjustBrightnessContrastFn
            ),
        ]
        .iter()
        .copied()
        .collect();
}

/// Adjusts brightness and contrast using opencv::convert_to() method.
/// 
/// * src: input matrix;
/// * dst: output matrix;
/// * alpha: scale factor.
/// * beta: delta added to the scaled values.
pub fn adjust_brightness_contrast_opencv(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    src.convert_to(dst, src.typ(), alpha, beta).unwrap();
}

#[inline]
fn adjust_value(val: u8, alpha: f64, beta: f64) -> u8 {
    use std::cmp;
    let val = (val as f64 * alpha + beta).round() as u8;
    cmp::min(cmp::max(val, 0), 255)
}

/// Adjusts brightness and contrast using sequential zip iteration 
/// through values of source and dest matrices.
/// 
/// * src: input matrix;
/// * dst: output matrix;
/// * alpha: scale factor.
/// * beta: delta added to the scaled values.
pub fn adjust_brightness_contrast_own(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let lut: Vec<u8> = (0..=255)
        .map(|val| adjust_value(val, alpha, beta))
        .collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.iter().zip(dst_data.iter_mut());
    it.for_each(|(src, dst)| unsafe { *dst = *lut.get_unchecked(*src as usize) });
}

/// Adjusts brightness and contrast using rayon-parallelized zip iteration 
/// through values of source and dest matrices.
/// 
/// * src: input matrix;
/// * dst: output matrix;
/// * alpha: scale factor.
/// * beta: delta added to the scaled values.
pub fn adjust_brightness_contrast_own_parallel(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let lut: Vec<u8> = (0..=255)
        .map(|val| adjust_value(val, alpha, beta))
        .collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.par_iter().zip(dst_data.par_iter_mut());
    it.for_each(|(src, dst)| unsafe { *dst = *lut.get_unchecked(*src as usize) });
}

/// Adjusts brightness and contrast using rayon-parallelized zip iteration 
/// through rows of source and dest matrices.
/// 
/// * src: input matrix;
/// * dst: output matrix;
/// * alpha: scale factor.
/// * beta: delta added to the scaled values.
pub fn adjust_brightness_contrast_own_parallel_rows(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let lut: Vec<u8> = (0..=255)
        .map(|val| adjust_value(val, alpha, beta))
        .collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let step = src.mat_step()[0];
    let src_iter = src_data.par_chunks(step);
    let dst_iter = dst_data.par_chunks_mut(step);
    let it = src_iter.zip(dst_iter);
    it.for_each(|(src, dst)| {
        let it = src.iter().zip(dst.iter_mut());
        it.for_each(|(src, dst)| unsafe {
             *dst = *lut.get_unchecked(*src as usize);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn test_func(func: AdjustBrightnessContrastFn) {
        let mut rng = rand::thread_rng();
        let h = rng.gen_range(1..400);
        let w = rng.gen_range(1..400);
        let alpha = rng.gen_range(0.0..2.0);
        let beta = rng.gen_range(-100.0..100.0);

        let mut src = unsafe { Mat::new_rows_cols(h, w, CV_8UC3).unwrap() };
        let low = Scalar::new(0., 0., 0., 0.);
        let high = Scalar::new(255., 255., 255., 255.);
        randu(&mut src, &low, &high).unwrap();

        let mut dst1 = unsafe { Mat::new_rows_cols(h, w, CV_8UC3).unwrap() };
        let mut dst2 = unsafe { Mat::new_rows_cols(h, w, CV_8UC3).unwrap() };
        adjust_brightness_contrast_opencv(&src, &mut dst1, alpha, beta);
        func(&src, &mut dst2, alpha, beta);
        let dst1 = dst1.data_bytes().unwrap();
        let dst2 = dst2.data_bytes().unwrap();

        assert!(dst1.iter().zip(dst2.iter()).all(|(a,b)| a == b), "Results are not equal");
    }

    #[test]
    fn test_own() {
        test_func(adjust_brightness_contrast_own);
    }

    #[test]
    fn test_own_parallel() {
        test_func(adjust_brightness_contrast_own_parallel);
    }

    #[test]
    fn test_own_parallel_rows() {
        test_func(adjust_brightness_contrast_own_parallel_rows);
    }
}
