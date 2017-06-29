extern crate num;
extern crate rexiv2;
extern crate chrono;

use rexiv2::Orientation;
use std::collections::HashSet;
use std::path::Path;
use jpegimpex::read_jpeg_size;
use chrono::datetime::DateTime;
use chrono::Local;
use serde::{self, Serialize, Serializer};

#[cfg(not(feature = "serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/types.rs"));

#[cfg(feature = "serde_macros")]
include!("types.rs.in");

impl serde::Deserialize for OrientationWrapper {
    fn deserialize<D>(deserializer: &mut D) -> Result<OrientationWrapper, D::Error>
        where D: serde::Deserializer
    {
        struct OrientationVisitor;

        impl serde::de::Visitor for OrientationVisitor {
            type Value = OrientationWrapper;
            fn visit_str<E>(&mut self, value: &str) -> Result<OrientationWrapper, E>
                where E: serde::de::Error
            {
                match value {
                   "Unspecified" =>  Ok(OrientationWrapper(Orientation::Unspecified)),
                   "Normal" =>  Ok(OrientationWrapper(Orientation::Normal)),
                   "HorizontalFlip" =>  Ok(OrientationWrapper(Orientation::HorizontalFlip)),
                   "Rotate180" =>  Ok(OrientationWrapper(Orientation::Rotate180)),
                   "VerticalFlip" =>  Ok(OrientationWrapper(Orientation::VerticalFlip)),
                   "Rotate190HorizontalFlip" =>  Ok(OrientationWrapper(Orientation::Rotate90HorizontalFlip)),
                   "Rotate90" =>  Ok(OrientationWrapper(Orientation::Rotate90)),
                   "Rotate90VerticalFlip" =>  Ok(OrientationWrapper(Orientation::Rotate90VerticalFlip)),
                   "Rotate270" =>  Ok(OrientationWrapper(Orientation::Rotate270)),
                   _ => Err(serde::de::Error::unknown_variant(value)),
                }
            }
        }

        deserializer.deserialize_struct_field(OrientationVisitor)
    }
}

fn get_exif_string(exiv: &rexiv2::Metadata, tags: &HashSet<String>, tag: &str) -> Option<String> {
    if tags.contains(tag) {
        match exiv.get_tag_string(tag) {
            Ok(e) => Some(e),
            _ => None
        }
    }
    else {
        None
    }
}

fn get_exif_rational(exiv: &rexiv2::Metadata, tags: &HashSet<String>, tag: &str) -> Option<num::rational::Ratio<i32>> {
    if tags.contains(tag) {
        exiv.get_tag_rational(tag)
    }
    else {
        None
    }
}

fn get_exif_multiple_strings(exiv: &rexiv2::Metadata, tags: &HashSet<String>, tag: &str) -> Vec<String> {
    if tags.contains(tag) {
        match exiv.get_tag_multiple_strings(tag) {
            Ok(e) => e,
            _ => Vec::new()
        }
    }
    else {
        Vec::new()
    }
}

fn get_digikam_color_label(exiv: &rexiv2::Metadata, tags: &HashSet<String>) -> Option<DigikamColorLabel> {
    let s = get_exif_string(&exiv, &tags, "Xmp.digiKam.ColorLabel");
    match s {
        Some(s) => match s.parse::<i32>() {
            Ok(number) => match number {
                0 => Some(DigikamColorLabel::None),
                1 => Some(DigikamColorLabel::Red),
                2 => Some(DigikamColorLabel::Orange),
                3 => Some(DigikamColorLabel::Yellow),
                4 => Some(DigikamColorLabel::Green),
                5 => Some(DigikamColorLabel::Blue),
                6 => Some(DigikamColorLabel::Magenta),
                7 => Some(DigikamColorLabel::Gray),
                8 => Some(DigikamColorLabel::Black),
                9 => Some(DigikamColorLabel::White),
                _ => None
            },
            _ => None
        },
        None => None
    }
}

fn get_digikam_pick_label(exiv: &rexiv2::Metadata, tags: &HashSet<String>) -> Option<DigikamPickLabel> {
    let s = get_exif_string(&exiv, &tags, "Xmp.digiKam.PickLabel");
    match s {
        Some(s) => match s.parse::<i32>() {
            Ok(number) => match number {
                0 => Some(DigikamPickLabel::None),
                1 => Some(DigikamPickLabel::Rejected),
                2 => Some(DigikamPickLabel::Pending),
                3 => Some(DigikamPickLabel::Accepted),
                _ => None
            },
            _ => None
        },
        None => None
    }
}

fn orientation_to_str(o: Orientation) -> &'static str { 
    match o {
        Orientation::Unspecified => "Unspecified",
        Orientation::Normal => "Normal",
        Orientation::HorizontalFlip => "HorizontalFlip",
        Orientation::Rotate180 => "Rotate180",
        Orientation::VerticalFlip => "VerticalFlip",
        Orientation::Rotate90HorizontalFlip => "Rotate90HorizontalFlip",
        Orientation::Rotate90 => "Rotate90",
        Orientation::Rotate90VerticalFlip => "Rotate90VerticalFlip",
        Orientation::Rotate270 => "Rotate270"
    }
}

impl Serialize for OrientationWrapper {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer
    {
        serializer.serialize_str(orientation_to_str(self.0))
    }
}

impl Metadata {
    pub fn from(file_path: &Path) -> Option<Metadata> {

        let size = read_jpeg_size(&file_path);

        let file_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                warn!("path '{}' contains invalid unicode", file_path.display());
                return None;
            }
        };

        let exif = match rexiv2::Metadata::new_from_path(file_str) {
            Ok(e) => Some(e),
            Err(err) => {
                warn!("image '{}' has no EXIF tags! Error: {}", file_path.display(), err);
                None
            }
        };

        let exif_tags = match exif.as_ref() {
            Some(exif) => match exif.get_exif_tags() {
                Ok(tags) => tags.into_iter().collect::<HashSet<_>>(),
                Err(err) => {
                    warn!("unicode error in EXIF tags: {}", err);
                    HashSet::<String>::new()
                }
            },
            None => HashSet::<String>::new()
        };

        let xmp_tags = match exif.as_ref() {
            Some(exif) => match exif.get_xmp_tags() {
                Ok(tags) => tags.into_iter().collect::<HashSet<_>>(),
                Err(err) => {
                    warn!("unicode error in XMP tags: {}", err);
                    HashSet::<String>::new()
                }
            },
            None => HashSet::<String>::new()
        };

        Some(Metadata {
            size: size,
            orientation: OrientationWrapper(match exif.as_ref() {
                Some(ref e) => e.get_orientation(),
                None => Orientation::Unspecified
            }),
            exposure_time: match exif.as_ref() {
                Some(ref e) => match e.get_exposure_time() {
                    Some(exposure) => Some(exposure),
                    None => None,
                },
                None => None,
            },
            iso_speed: match exif.as_ref() {
                Some(ref e) => e.get_iso_speed(),
                None => None
            },
            fnumber: match exif.as_ref() {
                Some(ref e) => {
                    let ratio = get_exif_rational(e, &exif_tags, "Exif.Photo.FNumber");
                    match ratio {
                        Some(fnumber) => Some(fnumber),
                        None => None,
                    }
                }
                None => None
            },
            lens_model: match exif.as_ref() {
                Some(ref e) => get_exif_string(e, &exif_tags, "Exif.Photo.LensModel"),
                None => None
            },
            camera_model: match exif.as_ref() {
                Some(ref e) => get_exif_string(e, &exif_tags, "Exif.Image.Model"),
                None => None
            },
            rating: match exif.as_ref() {
                Some(ref e) => {
                    let rating_str = get_exif_string(&e, &xmp_tags, "Xmp.xmp.Rating");
                    match rating_str {
                        Some(s) => match s.parse::<i32>() {
                            Ok(i) => Some(i),
                            Err(err) => {
                                warn!("expected an integer rating, got '{}': {}", s, err);
                                None
                            }
                        },
                        None => None
                    }
                },
                None => None
            },
            tags: match exif.as_ref () {
                Some(ref e) =>  get_exif_multiple_strings(e, &xmp_tags, "Xmp.digiKam.TagsList"),
                None => Vec::new()
            },
            digikam_pick_label: match exif.as_ref() {
                Some(ref e) => get_digikam_pick_label(e, &xmp_tags),
                None => None
            },
            digikam_color_label: match exif.as_ref() {
                Some(ref e) => get_digikam_color_label(e, &xmp_tags),
                None => None
            }
        })
    }
}
