extern crate time;
extern crate imageproc;

use std::path::Path;
use time::PreciseTime;
use imageproc::convert::*;
use imageproc::transform;
use imageproc::image::*;
use imageproc::conv::*;
use imageproc::imageio::{ImageIO, FreeImageIO};

#[allow(unused_variables)] 
#[allow(dead_code)]
fn main() {
    println!("start perf...");

    let _: Result<ImageBgra, _> = FreeImageIO::from_path(&Path::new("XX"));
    let mut src = ImageBgra::new(2000,1000);
    src.fill(&Bgra([100,100,100,255]));
    // XXX link problem!
    let out = convert::<MapBgraGray>(&src);
    let out = transform::resize_nearest(&src, 4000, 2000);

    let start = PreciseTime::now();

    let src = vec![1f32; 10000];
    let kern: Vec<f32> = vec![1.0, 2.0, 2.0, 4.0, 5.0, 4.0, 2.0, 2.0, 1.0];
    let mut out = vec![0f32; src.len() as usize];
    for _ in 0..1000 {
        conv1d(&src, &mut out, &kern);
    }
    let end = PreciseTime::now();
    println!("finished in {} ms", start.to(end).num_milliseconds());
}
