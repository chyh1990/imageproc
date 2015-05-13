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

    let mut src = ImageBgra::new(200,100);

    let start = PreciseTime::now();

    for _ in 0..1000 {
        let _ = split(&src);
    }
    let end = PreciseTime::now();
    println!("finished in {} ms", start.to(end).num_milliseconds());
}
