use opencv::core::*;


pub fn adjust_brightness_contrast_opencv(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let typ = dst.typ();
    src.convert_to(dst, typ, alpha, beta).unwrap();
}



fn adjust_value(val: u8, alpha: f64, beta: f64) -> u8 {
    use std::cmp;
    let val = (val as f64 * alpha + beta) as u8;
    cmp::min(cmp::max(val, 0), 255)
}

pub fn adjust_brightness_contrast_own(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let height = src.rows() as isize;
    let width = src.cols();
    let src_step = src.mat_step()[0] as isize;
    let dst_step = dst.mat_step()[0] as isize;
    for y in 0..height {
        unsafe {
            let mut src_p = src.data().offset(y * src_step);
            let mut dst_p = dst.data_mut().offset(y * dst_step);
            for _x in 0..width {
                *dst_p = adjust_value(*src_p, alpha, beta);
                src_p = src_p.offset(1);
                dst_p = dst_p.offset(1);
                *dst_p = adjust_value(*src_p, alpha, beta);
                src_p = src_p.offset(1);
                dst_p = dst_p.offset(1);
                *dst_p = adjust_value(*src_p, alpha, beta);
                src_p = src_p.offset(1);
                dst_p = dst_p.offset(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
