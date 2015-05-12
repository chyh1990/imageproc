use image::*;
use math::utils::*;
use traits::Primitive;

pub fn conv1d<T: Primitive>(row: &[T], out: &mut [f32], kernel: &[f32]) {
    assert!(row.len() <= out.len());
    let hx = kernel.len() as i32 / 2;
    let w = row.len() as i32;
    for i in 0..w {
        let mut s = 0f32;
        for j in 0..kernel.len() as i32 {
            let xi = clip((i - hx) + j, 0, w - 1) as usize;
            s += row[xi].to_f32().unwrap() * kernel[j as usize];
        }
        out[i as usize] = s;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_conv1d() {
        let src: Vec<f32> = vec!(1.0, 1.0, 1.0, 2.0, 2.0, 2.0);
        let kern: Vec<f32> = vec!(1.0, 2.0, 1.0);
        let res: Vec<f32> = vec![4.0, 4.0, 5.0, 7.0, 8.0, 8.0];
        let mut out = [0f32; 6];
        conv1d(&src, &mut out, &kern);
        assert_eq!(res, out);
    }
}

