use image::*;

#[inline(always)]
fn clipped_round(x: f32, min: i32, max: i32) -> i32 {
    let t = x.round() as i32;
    if t < min {
        return min;
    }
    if t > max {
        return max;
    }
    t
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


#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;
    use image::*;
    use imageio::ImageIO;
    use imageio::FreeImageIO;

    #[test]
    fn test_resize() {
        let path = Path::new("./tests/cat.jpg");
        let img = FreeImageIO::from_path(&path).unwrap();

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
}

