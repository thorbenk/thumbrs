#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

#[macro_use]
extern crate log;

extern crate chrono;
extern crate filetime;
extern crate image;
extern crate libc;
extern crate mozjpeg_sys;
extern crate num;
extern crate rexiv2;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate sha1;
extern crate walkdir;

pub mod jpegimpex;
pub mod metadata;
pub mod thumbnail;

pub use jpegimpex::{read_jpeg, write_jpeg};
pub use metadata::{Metadata, FileInfo};
pub use thumbnail::{make_thumbnail, read_and_rotate};
