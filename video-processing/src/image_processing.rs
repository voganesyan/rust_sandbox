use opencv::core::*;


pub fn adjust_brightness_contrast_opencv(src: &Mat, dst: &mut Mat, alpha: f64, beta: f64) {
    let typ = dst.typ();
    src.convert_to(dst, typ, alpha, beta).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
