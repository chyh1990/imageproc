# ImageProc

[![Build Status](https://travis-ci.org/chyh1990/imageproc.svg?branch=master)](https://travis-ci.org/chyh1990/imageproc)

Maintainers: @chyh1990

**imageproc** is a advanced image proccessing library for the Rust language in order to provide:

* generic pixel, image, geometry data structure alike OpenCV
* Image IO for variable image file format
* image transformations
* image processing routines, e.g. convolution, gaussian blur, etc.
* canvas and rasterization (TBD)

This library is inspired by the following projects:

* [OpenCV](http://opencv.org/)
* [PistonDevelopers/image](https://github.com/PistonDevelopers/image)


## Usage

Adding the following to the Cargo.toml in your project:

```
[dependencies.imageproc]
git = "https://github.com/chyh1990/imageproc.git"
```

and import using *extern crate*:

```.rust
extern crate imageproc;
```

Most data structures and routines are exposed uder **imageproc** prefix:

```.rust
extern crate imageproc;

use std::path::Path;
use imageproc::image::*;
use imageproc::conv::*;
use imageproc::imageio::{ImageIO, FreeImageIO}; 

fn main() {
	let img: ImageBgra = FreeImageIO::from_path(&Path::new("cat.jpg"));
	let out: conv::gaussian_blur(&img, 11, 0f32);

	let target = Path::new("out.png");
	FreeImageIO::save(&target, &out).unwrap();
}

``` 

### Supported Image Format

**imageproc** use cross-platform native library to decode/encode images. The only supported backend
is [FreeImage](http://freeimage.sourceforge.net/), it includes decoders for most image formats, and encoders
for most common used formats.

## Contribution

Fork & pull request on Github.

