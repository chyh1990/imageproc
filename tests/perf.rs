extern crate time;
extern crate imageproc;

use time::PreciseTime;
use imageproc::convert::*;
use imageproc::transform;
use imageproc::image::*;

#[allow(unused_variables)] 
#[allow(dead_code)]
fn main() {
    println!("start perf...");

    let mut src = ImageBGRA::new(2000,1000);
    src.fill(&Bgra([100,100,100,255]));
    let start = PreciseTime::now();
    for _ in 0..10 {
        // let out = convert::<MapBGRA_Gray>(&src);
        // let out = transform::resize_nearest(&src, 4000, 2000);
        let out = transform::resize_bilinear(&src, 4000, 2000);
    }
    let end = PreciseTime::now();
    println!("finished in {} ms", start.to(end).num_milliseconds());
}
