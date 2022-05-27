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

pub fn adjust_brightness_contrast_opencv(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    src.convert_to(dst, src.typ(), alpha, beta).unwrap();
}

#[inline]
fn adjust_value(val: u8, alpha: f64, beta: f64) -> u8 {
    use std::cmp;
    let val = (val as f64 * alpha + beta) as u8;
    cmp::min(cmp::max(val, 0), 255)
}

pub fn adjust_brightness_contrast_own(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let lut: Vec<u8> = (0..=255)
        .map(|val| adjust_value(val, alpha, beta))
        .collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.iter().zip(dst_data.iter_mut());
    it.for_each(|(src, dst)| *dst = lut[*src as usize]);
}

pub fn adjust_brightness_contrast_own_parallel(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let lut: Vec<u8> = (0..=255)
        .map(|val| adjust_value(val, alpha, beta))
        .collect();
    let src_data = src.data_bytes().unwrap();
    let dst_data = dst.data_bytes_mut().unwrap();
    let it = src_data.par_iter().zip(dst_data.par_iter_mut());
    it.for_each(|(src, dst)| *dst = lut[*src as usize]);
}

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
        it.for_each(|(src, dst)| {
            *dst = lut[*src as usize];
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
