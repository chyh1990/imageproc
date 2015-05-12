use image::*;
use num::NumCast;
use math::utils::*;
use num::traits::ToPrimitive;
use std::ptr;
use traits::Primitive;

#[inline]
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

pub fn conv2d_sep<T: Pixel>(src: &Image<T>, kernelx: &[f32], kernely: &[f32]) -> Image<T> 
{
    let mut dst: Image<T> = Image::new(src.width(), src.height());
    let height = src.height();
    let width = src.width();
    let channels = T::channels() as usize;
    let mut row_off = vec![0u32; kernely.len()];

    let hkxw = kernelx.len() / 2;
    let tmpsz = channels * (width as usize + kernelx.len() + 1);
    let mut tmp = vec![0f32; tmpsz];
    for y in 0..height {
        let pdst = dst.row_mut(y);
        for i in 0..kernely.len() {
            let yy = y as i32 - kernely.len() as i32 / 2 + i as i32;
            row_off[i] = clip(yy, 0, height as i32 - 1) as u32;
        }
        unsafe { ptr::write_bytes(tmp.as_mut_ptr(), 0, tmpsz); }
        for x in 0..width {
            let tx = (x as usize + hkxw) * channels;
            for i in 0..kernely.len() {
                let r = src.row(row_off[i]);
                for c in 0..channels {
                    tmp[tx + c] += r[x as usize].raw()[c].to_f32().unwrap() * kernely[i];
                }
            }
        }
        for x in 0..hkxw {
            let tx = x * channels;
            let tx1 = (x + width as usize + hkxw) * channels;
            for c in 0..channels {
                tmp[tx + c] = tmp[hkxw * channels + c];
                tmp[tx1 + c] = tmp[(hkxw + width as usize - 1) * channels + c];
            }
        }
        for x in 0..width {
            // XXX max channel?
            let mut px = [0f32; 4];
            for i in 0..kernelx.len() {
                let tx = (x as usize + hkxw - kernelx.len() / 2 + i) * channels;
                for c in 0..channels {
                    px[c] += tmp[tx + c] * kernelx[i];
                }
            }
            for c in 0..channels {
                pdst[x as usize].raw_mut()[c] = NumCast::from(px[c]).unwrap();
            }
        }
    }
    dst
}

fn gaussian_kernel(w: usize, sigma: f32) -> Vec<f32> {
    let hw = (w + 1) / 2;
    let sigma: f32 = match sigma {
        0f32...0.00001f32 => 0.3 * ((w as f32 - 1.0) * 0.5 - 1.0) + 0.8,
        _ => sigma
    };
    //sigma = 0.3 * ((w - 1) * 0.5 - 1) + 0.8;
    let mut k = vec![0f32; w];
    let s2 = sigma * sigma;
    for i in 0..hw {
        let t = (hw - i) as f32;
        k[i] = (-t * t / 2.0 / s2).exp_m1() + 1f32;
    }
    for i in hw..w {
        k[i] = k[w - i - 1];
    }
    let mut s = 0f32;
    for i in 0..w { s += k[i]; }
    for i in 0..w { k[i] /= s; }
    k
}

pub fn gaussian_blur<T: Pixel>(src: &Image<T>, kernel_width :usize, sigma: f32) -> Image<T> {
    assert!(kernel_width >= 1);
    let k = gaussian_kernel(kernel_width, sigma);
    conv2d_sep(src, &k, &k)
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use image::*;
    use super::*;
    use imageio::ImageIO;
    use imageio::FreeImageIO;

    #[test]
    fn test_conv1d() {
        let src: Vec<f32> = vec!(1.0, 1.0, 1.0, 2.0, 2.0, 2.0);
        let kern: Vec<f32> = vec!(1.0, 2.0, 1.0);
        let res: Vec<f32> = vec![4.0, 4.0, 5.0, 7.0, 8.0, 8.0];
        let mut out = [0f32; 6];
        conv1d(&src, &mut out, &kern);
        assert_eq!(res, out);
    }

    #[test]
    fn test_conv2d_sep() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();

        let out = gaussian_blur(&img, 11, 0f32);

        let target = Path::new("/tmp/test-conv-out1.jpg");
        FreeImageIO::save(&target, &out).unwrap();
    }
}

