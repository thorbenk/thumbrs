use image;
use image::{GenericImage, DynamicImage, ImageRgba8};
use image::imageops::*;

use rexiv2;
use rexiv2::Orientation;

use std::path::Path;
use std::cmp::Ordering;

use jpegimpex::{read_jpeg, write_jpeg};

fn write_resized_image (
    img: &DynamicImage,
    w: u32,
    h: u32,
    quality: u8,
    output_filename: &Path
) {
    let thumb = image::imageops::resize(img, w, h, image::CatmullRom);

    let out:DynamicImage = ImageRgba8(thumb);
    
    write_jpeg (&out, output_filename, quality);
}

pub fn read_and_rotate (
    img_filename: &Path
) -> DynamicImage {
    let exif_orientation = rexiv2::Metadata::new_from_path(img_filename.to_str().unwrap())
        .map(|e| e.get_orientation());

    let mut img = read_jpeg(img_filename);

    match exif_orientation {
        Ok(e) => match e {
            Orientation::Unspecified => (),
            Orientation::Normal => (),
            Orientation::HorizontalFlip => {
                img = img.fliph();
            }
            Orientation::Rotate180 => {
                img = img.rotate180();
            },
            Orientation::VerticalFlip => {
                img = img.flipv();
            },
            Orientation::Rotate90HorizontalFlip => {
                img = img.rotate90().fliph();
            },
            Orientation::Rotate90 => {
                img = img.rotate90();
            }
            Orientation::Rotate90VerticalFlip => {
                img = img.rotate90().flipv();
            },
            Orientation::Rotate270 => {
                img = img.rotate270();
            }
        },
        Err(_) => ()
    };
    
    img
}

pub fn make_thumbnail (
    img: &DynamicImage,
    size: u32,
    quality: u8,
    out_abspath: &Path) -> (u32, u32) {

    let aspect = (img.width() as f64) / (img.height() as f64);
    let s = size.clone() as f64;
    let (w,h) = match img.width().cmp(&img.height()) {
        Ordering::Greater => (size.clone(), (1.0/aspect * s) as u32),
        Ordering::Less => ((aspect * s ) as u32, size.clone()),
        Ordering::Equal => (size.clone(), size.clone())
    };

    let thumb_file = format!("{}_{}x{}.jpg", out_abspath.file_name().unwrap().to_str().unwrap(), w, h);
    let thumb_path = out_abspath.parent().unwrap().join(Path::new(&thumb_file));

    write_resized_image (&img, w, h, quality, &thumb_path);

    (w,h)
}
