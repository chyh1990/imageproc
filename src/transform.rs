use image::*;
use num::NumCast;
use std::ops::Index;
use num::traits::Bounded;
use math::utils::*;
use math::affine::Affine2D;
use num::traits::ToPrimitive;

pub enum InterplateType {
    Nearest,
    Bilinear
}

pub fn resize_nearest<T: Pixel>(src: &Image<T>, width: u32, height: u32) -> Image<T> {
    let mut dst = Image::new(width, height);
    let yscale: f32 = src.height() as f32 / height as f32;
    let xscale: f32 = src.width() as f32 / width as f32;
    let mut xidx: Vec<usize> = Vec::with_capacity(width as usize);
    for w in 0..width as usize {
        xidx.push(clipped_round(w as f32 * xscale, 0,
                src.width() as i32 - 1) as usize);
    }
    for h in 0..height {
        let pdst = dst.row_mut(h);
        let psrc = src.row(clipped_round(h as f32 * yscale, 0,
                src.height() as i32 - 1) as u32);
        for w in 0..width as usize {
            pdst[w] = psrc[xidx[w]];
        }
    }
    dst
}

pub fn resize_bilinear<T: Pixel>(src: &Image<T>, width: u32, height: u32) -> Image<T> {
    let mut dst = Image::new(width, height);
    let yscale: f32 = src.height() as f32 / height as f32;
    let xscale: f32 = src.width() as f32 / width as f32;

    let mut x_0: Vec<usize> = Vec::with_capacity(width as usize);
    let mut x_1: Vec<usize> = Vec::with_capacity(width as usize);
    let mut d_0: Vec<f32> = Vec::with_capacity(width as usize);

    for w in 0..width as usize {
        let mid = w as f32 * xscale;
        let l = mid.floor();
        let r = mid.ceil();
        let d = r - mid;
        x_0.push(clipped_round(l, 0, src.width() as i32 - 1) as usize);
        x_1.push(clipped_round(r, 0, src.width() as i32 - 1) as usize);
        d_0.push(d);
    }
    for h in 0..height {
        let pdst = dst.row_mut(h);

        let mid = h as f32 * yscale;
        let t = mid.floor();
        let b = mid.ceil();
        let dy = b - mid;

        let psrc0 = src.row(clipped_round(t, 0, src.height() as i32 - 1) as u32);
        let psrc1 = src.row(clipped_round(b, 0, src.height() as i32 - 1) as u32);
        for w in 0..width as usize {
            let x0 = x_0[w];
            let x1 = x_1[w];
            //let a = psrc0[x0].blend(psrc0[x1], d_0[w]);
            //let b = psrc1[x0].blend(psrc1[x1], d_0[w]);
            //pdst[w] = a.blend(b, dy);
            pdst[w] = psrc0[x0].blend4(
                psrc0[x1],
                psrc1[x0],
                psrc1[x1],
                d_0[w], dy);
        }
    }
    dst
}

pub fn resize<T: Pixel>(src: &Image<T>, width: u32, height: u32, interp: InterplateType) -> Image<T> {
    match interp {
        InterplateType::Nearest => resize_nearest(src, width, height),
        InterplateType::Bilinear => resize_bilinear(src, width, height)
    }
}

pub fn warp_perspective<T: Pixel>(src: &Image<T>, width: u32, height: u32, affine :&Affine2D, interp :InterplateType) -> Image<T> {
    let mut dst: Image<T> = Image::new(width, height);
    for h in 0..height {
        let pdst = dst.row_mut(h);
        for w in 0..width {
            let coord = affine.apply_inv([w as f32, h as f32, 1f32]);
            let sx = coord[0] / coord[2];
            let sy = coord[1] / coord[2];
            match interp {
                InterplateType::Nearest => {
                    let ix = sx.round() as i32;
                    let iy = sy.round() as i32;
                    if ix >= 0 && ix < src.width() as i32 
                        && iy >= 0 && iy < src.height() as i32 {
                            pdst[w as usize] = src[(ix as u32, iy as u32)];
                        }
                },
                InterplateType::Bilinear => {
                    let u = sx.ceil() - sx;
                    let v = sy.ceil() - sy;
                    let x0 = clip(sx.floor() as i32, 0, src.width() as i32 - 1) as u32;
                    let y0 = clip(sy.floor() as i32, 0, src.height() as i32 -1) as u32;
                    let x1 = clip(sx.ceil() as i32, 0, src.width() as i32 -1) as u32;
                    let y1 = clip(sy.ceil() as i32, 0, src.height() as i32 - 1) as u32;
                    pdst[w as usize] = src[(x0, y0)].blend4(
                        src[(x1, y0)],
                        src[(x0, y1)],
                        src[(x1, y1)],
                        u, v);
                }
            }
        }
    }
    dst
}

pub fn flip_vertical<T: Pixel>(src: &Image<T>) -> Image<T> {
    let mut dst = Image::new(src.width(), src.height());
    for h in 0..src.height() {
        for (a, b) in dst.row_mut(src.height() - h - 1).iter_mut().zip(src.row(h)) {
            *a = *b;
        }
    }
    dst
}

pub fn flip_horizontal<T: Pixel>(src: &Image<T>) -> Image<T> {
    let mut dst = Image::new(src.width(), src.height());
    for h in 0..src.height() {
        for (a, b) in dst.row_mut(h).iter_mut().zip(src.row(h).iter().rev()) {
            *a = *b;
        }
    }
    dst
}

pub fn min<T, U>(src: &Image<T>) -> U
    where T: Pixel,
          U: Pixel,
          T: Index<usize, Output=U::Subpixel>
{
    let mut t = [U::Subpixel::max_value(); MAX_CHANNEL_COUNT];
    for (_, _, p) in src.iter() {
        for c in 0..T::channels() {
            if t[c] > p[c] {
                t[c] = p[c];
            }
        }
    }
    U::from_raw(&t)
}

pub fn max<T, U>(src: &Image<T>) -> U
    where T: Pixel,
          U: Pixel,
          T: Index<usize, Output=U::Subpixel>
{
    let mut t = [U::Subpixel::min_value(); MAX_CHANNEL_COUNT];
    for (_, _, p) in src.iter() {
        for c in 0..T::channels() {
            if t[c] < p[c] {
                t[c] = p[c];
            }
        }
    }
    U::from_raw(&t)
}

pub fn normalize<U, V, M>(src: &Image<U>, alpha: f32, beta: f32) -> Image<V> 
    where U: Pixel,
          V: Pixel,
          M: Pixel,
          U: Index<usize, Output=M::Subpixel>
{
    let mut dst = Image::<V>::new(src.width(), src.height());
    let mut mins = [0f32; MAX_CHANNEL_COUNT];
    let mut maxs = [0f32; MAX_CHANNEL_COUNT];
    let min_p: M = min(&src);
    let max_p: M = max(&src);
    let s = beta - alpha;
    for c in 0..U::channels() {
        mins[c] = min_p.raw()[c].to_f32().unwrap();
        maxs[c] = max_p.raw()[c].to_f32().unwrap();
    }
    for ((_, _, p), (_, _, q)) in dst.iter_mut().zip(src.iter()) {
        for c in 0..U::channels() {
            let d = maxs[c] - mins[c];
            let t = (q.raw()[c].to_f32().unwrap() - mins[c]) / d * s + alpha;
            p.raw_mut()[c] = NumCast::from(t).unwrap();
        }
    }
    dst
}

//pub fn max<T: Pixel>(src: &Image<T>) -> (u32, u32, T) {
//}

#[derive(Debug, Clone)]
pub enum RotateType{
    Cw0,
    Cw90,
    Cw180,
    Cw270
}

pub fn rotate_cw90<T: Pixel>(src: &Image<T>) -> Image<T> {
    let mut dst = Image::new(src.height(), src.width());
    for h in 0..src.height() {
        let psrc = src.row(h);
        let k = (src.height() - h - 1) as usize;
        for w in 0..src.width() {
            dst.row_mut(w)[k] = psrc[w as usize];
        }
    }
    dst
}

pub fn rotate_cw180<T: Pixel>(src: &Image<T>) -> Image<T> {
    let mut dst = Image::new(src.width(), src.height());
    for h in 0..src.height() {
        let psrc = src.row(h);
        let pdst = dst.row_mut(src.height() - h - 1);
        for w in 0..src.width() as usize {
            pdst[w] = psrc[src.width() as usize - w - 1];
        }
    }

    dst
}

pub fn rotate_cw270<T: Pixel>(src: &Image<T>) -> Image<T> {
    let mut dst = Image::new(src.height(), src.width());
    for h in 0..src.height() {
        let psrc = src.row(h);
        for w in 0..src.width() {
            dst.row_mut(src.width() - w - 1)[h as usize] = psrc[w as usize];
        }
    }
    dst
}

fn rotate<T: Pixel>(src: &Image<T>, rtype: RotateType) -> Image<T> {
    match rtype {
        RotateType::Cw0 => src.clone(),
        RotateType::Cw90 => rotate_cw90(src),
        RotateType::Cw180 => rotate_cw180(src),
        RotateType::Cw270 => rotate_cw270(src)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;
    use image::ImageBgra;
    use imageio::ImageIO;
    use imageio::FreeImageIO;
    use geo::*;
    use math::affine::Affine2D;

    #[test]
    fn test_resize() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();

        let dst = resize_nearest(&img, 600, 400);
        assert_eq!(dst.width(), 600);
        assert_eq!(dst.height(), 400);
        let target = Path::new("/tmp/test-resize-out1.jpg");
        FreeImageIO::save(&target, &dst).unwrap();

        let dst = resize_bilinear(&img, 600, 400);
        assert_eq!(dst.width(), 600);
        assert_eq!(dst.height(), 400);
        let target = Path::new("/tmp/test-resize-out2.jpg");
        FreeImageIO::save(&target, &dst).unwrap();
    }

    #[test]
    fn test_warp() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();
        let src = vec![Pointf::new(0f32, 0f32), Pointf::new(1f32, 0f32)];
        let dst = vec![Pointf::new(0f32, 0f32), Pointf::new(1f32, 1f32)];

        let aff = Affine2D::nonreflect_similarity_from_points(&src, &dst).unwrap();
        let out = warp_perspective(&img, 200, 200, &aff, InterplateType::Nearest);
        let target = Path::new("/tmp/test-affine-out1.jpg");
        FreeImageIO::save(&target, &out).unwrap();

        let out = warp_perspective(&img, 300, 300, &aff, InterplateType::Bilinear);
        let target = Path::new("/tmp/test-affine-out2.jpg");
        FreeImageIO::save(&target, &out).unwrap();
    }

    #[test]
    fn test_flip() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();

        let out = flip_vertical(&img);
        let target = Path::new("/tmp/test-flip-out1.jpg");
        FreeImageIO::save(&target, &out).unwrap();

        let out = flip_horizontal(&img);
        let target = Path::new("/tmp/test-flip-out2.jpg");
        FreeImageIO::save(&target, &out).unwrap();
    }

    #[test]
    fn test_rotate() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();

        let out = rotate_cw90(&img);
        let target = Path::new("/tmp/test-rotate-out1.jpg");
        FreeImageIO::save(&target, &out).unwrap();

        let out = rotate_cw180(&img);
        let target = Path::new("/tmp/test-rotate-out2.jpg");
        FreeImageIO::save(&target, &out).unwrap();

        let out = rotate_cw270(&img);
        let target = Path::new("/tmp/test-rotate-out3.jpg");
        FreeImageIO::save(&target, &out).unwrap();
    }
}

