#![cfg_attr(nightly, feature(custom_derive, plugin))]
#![cfg_attr(nightly, plugin(serde_macros))]
#![cfg_attr(nightly, plugin(docopt_macros))]

extern crate walkdir;
extern crate filetime;
extern crate sha1;
extern crate chrono;
extern crate rustc_serialize;
extern crate image;
#[macro_use]
extern crate log;
extern crate rexiv2;
extern crate num;
extern crate serde;
extern crate serde_json;
extern crate docopt;

extern crate thumbrs;
use thumbrs::*;

use std::io::{self};
use std::io::prelude::*;

use std::fs::{self, File};

use std::path::Path;

use filetime::FileTime;

use chrono::datetime::DateTime;
use chrono::naive::datetime::NaiveDateTime;
use chrono::offset::TimeZone;
use chrono::Local;

use docopt::Docopt;

use std::thread;
use std::sync::{Arc, mpsc};

// Iterate through `iter` while it matches `prefix`; return `None` if `prefix`
// is not a prefix of `iter`, otherwise return `Some(iter_after_prefix)` giving
// `iter` after having exhausted `prefix`.
fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I> where
    I: Iterator<Item=A> + Clone, J: Iterator<Item=A>, A: PartialEq
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(x), Some(y)) => {
                if x != y { return None }
            }
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}

fn strip_prefix<'a> (path: &'a Path, base: &'a Path) -> Option<&'a Path> {
    iter_after(path.components(), base.components()).map(|c| c.as_path())
}

#[allow(dead_code)]
fn file_sha1 (path: &Path) -> io::Result<String> {
    let mut f = try!(File::open(path));
    let mut buffer: Vec<u8> = Vec::new();
    let _ = try!(f.read_to_end(&mut buffer));
    let mut s = sha1::Sha1::new();
    s.update(&buffer);
    return Ok(s.hexdigest());
}

#[allow(dead_code)]
fn get_mtime(metadata: &Result<fs::Metadata, std::io::Error>) -> DateTime<Local> {
    let mtime = metadata.as_ref().map(|meta| {
        FileTime::from_last_modification_time(&meta)
    }).unwrap_or(FileTime::zero());

    let naive_datetime = NaiveDateTime::from_timestamp(mtime.seconds_relative_to_1970() as i64, mtime.nanoseconds());
    Local.from_local_datetime (&naive_datetime).single()
     .unwrap_or(Local::now())
}

fn walk_filetree(input_path: &Path, output_path: &Path, no_thumbs: bool) {
    walk_filetree_impl(&input_path, &output_path, no_thumbs, Vec::new());
}

fn is_dir (entry: &fs::DirEntry) -> bool {
    // let hidden_dirs = ["0-sterne", "raw"];
    entry.metadata().unwrap().is_dir()
}

fn is_image (entry: &fs::DirEntry) -> bool {
    let image_extensions = ["JPG", "jpg"];
    let path = entry.path();
    let ext = match path.extension() {
        Some(e) => e.to_str().unwrap().clone(),
        None => ""
    };
    image_extensions.iter().any(|x| *x == ext)
}

fn tree_prefix (ancestor_at_end: &Vec<bool>) -> String {
    ancestor_at_end
        .iter()
        .map(|a| match *a {
            true =>  "│   ",
            false => "    "
            })
        .collect::<String>()
}

fn tree_line (progress: Option<(u32, u32)>, ancestor_at_end: &Vec<bool>, has_subcontent: bool, suffix: &str) -> String {
    let s = match progress {
        Some(p) => format!("{:02}/{:02} ", p.0, p.1),
        None => "      ".to_string()
    };

    s + &tree_prefix(ancestor_at_end)
      + if has_subcontent { "├── " } else { "└── " }
      + suffix
}

fn walk_filetree_impl(input_path: &Path, output_path: &Path, no_thumbs: bool, ancestor_at_end: Vec<bool>) {
    let dir_iter = match fs::read_dir(input_path) {
        Ok(i) => i,
        Err(_) => {
            return;
        }
    };

    let mut dir_contents = dir_iter
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    dir_contents
        .sort_by(|a, b| {
            let a = a.path();
            let b = b.path();
            a.cmp(&b)
        });

    let sub_dirs = dir_contents.iter()
        .filter(|e| is_dir(*e))
        .collect::<Vec<_>>();

    let jpegs = dir_contents.iter()
        .filter(|e| !e.metadata().unwrap().is_dir())
        .filter(|e| is_image(*e))
        .collect::<Vec<_>>();

    if sub_dirs.len() == 0 && jpegs.len() == 0 {
        return;
    }

    fs::create_dir_all(output_path)
        .ok()
        .expect("Could not create output dir");

    let mut generation_infos = Vec::<FileInfo>::new();

    let sizes: Vec<u32> = vec![100, 200, 300, 640, 800, 1024, 1920];
    let qualities: Vec<u8> = vec![75, 75, 75, 88, 88, 88, 88];
    assert!(sizes.len() == qualities.len());
    let thumbnail_count = sizes.len();

    let jpeg_count = jpegs.len();
    for (i, curr_entry) in jpegs.iter().enumerate() {

        // absolute path to source image
        let in_abspath = curr_entry.path();

        // filename of source image
        // TODO: can simplify to filename() of path or sth.
        let in_fname = strip_prefix(&in_abspath, &input_path).unwrap();

        // filename of output image, corresponding to source image
        // (we will append suffixes to this for different thumbnail sizes)
        let out_abspath = output_path.join(in_fname);
        
        let has_subcontent = i < jpeg_count - 1;

        let update_line = |curr_i: u32| {
            let t = tree_line(Some((curr_i as u32, (thumbnail_count+1) as u32)), &ancestor_at_end, has_subcontent, in_fname.to_str().unwrap());
            let _ = std::io::stdout().write( (String::new() + "\r" + &t).as_bytes());
            let _ = std::io::stdout().flush();
        };

        let m = Metadata::from(&in_abspath);

        ////

        let thumbnail_sizes = match no_thumbs {
            true => Vec::<(u32,u32)>::new (),
            false => {
                let mut tsizes = Vec::<(u32,u32)>::new ();
                let (tx, rx) = mpsc::channel();

                let img = read_and_rotate (&in_abspath);

                update_line (1);

                let shared_img = Arc::new(img);

                for (size, quality) in sizes.iter().cloned().zip(qualities.iter().cloned()) {
                    let tx = tx.clone();

                    let out = out_abspath.clone ();

                    let local_img = shared_img.clone();
                    thread::spawn(move || {
                        //println!("MAKE THUMB {} {} {:?}", size, quality, &out);
                        let (w,h) = make_thumbnail (&local_img, size, quality, &out);
                        tx.send((w, h)).unwrap();
                    });
                }

                for i in 0..thumbnail_count {
                    let (w,h) = rx.recv().unwrap();

                    update_line ((i+2) as u32);

                    tsizes.push((w, h));
                }
                tsizes
            }
        };

        let _ = std::io::stdout().write("\n".as_bytes());
        let _ = std::io::stdout().flush();

        let mtime = get_mtime(&curr_entry.metadata());
        let hexdigest = file_sha1(&curr_entry.path())
            .ok()
            .expect("Could not compute SHA1 sum");

        let timestamp = Timestamp(mtime); 
        let file_info = FileInfo { filename: in_abspath.to_str().unwrap().to_string(), sha1sum: hexdigest, modified_time: timestamp, metadata: m.unwrap(), thumbnail_sizes: thumbnail_sizes };

        generation_infos.push(file_info);
    }


    if generation_infos.len() > 0 {
        let j = serde_json::to_string_pretty(&generation_infos).unwrap();

        let f = "_".to_string() + input_path.file_name().unwrap().to_str().unwrap() + ".json";

        let msg = String::new() + "{meta: " + &f + "}";
        println!("      {}{}", tree_prefix(&ancestor_at_end), &msg);
       
        let json_file = output_path.join(f);
        let mut f = File::create(json_file).unwrap();
        match f.write_all(j.as_bytes()) {
            Ok (_) => (),
            Err (_) => warn!("Error writing json to disk")
        }
    }

    let subdir_count = sub_dirs.len();
    for (i, dir) in sub_dirs.iter().enumerate() {
        let path = dir.path();
        let relative_file = strip_prefix(&path, &input_path).unwrap();
        let out_file = output_path.join(relative_file);

        if fs::read_dir(&path).is_ok() {
            let has_subcontent = i < subdir_count-1;
            let t = tree_line(None, &ancestor_at_end, has_subcontent, relative_file.to_str().unwrap());
            println!("{}", t);
            let mut a = ancestor_at_end.clone();
            a.push(has_subcontent);
            walk_filetree_impl(&dir.path(), &out_file, no_thumbs, a);
        }
        else {
            let has_subcontent = false;
            let suffix : String = String::new() + relative_file.to_str().unwrap() + " [inaccessible]";
            let t = tree_line(None, &ancestor_at_end, has_subcontent, &suffix);
            println!("{}", t);
        }

    }
}

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_inpath: String,
    arg_outpath: String,
    flag_help: bool,
    flag_no_thumbs: bool,
}

const USAGE: &'static str = "
Thumbnail and image metadata extractor.

Usage:
  thumbrs [-d] <inpath> <outpath>

Options:
  -h --help       Show this screen.
  -d --no-thumbs  Do not generate thumbnails (but extract metadata).
";

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    println!("Rust thumbnail and meta-data extractor.");
    println!("");

    let inpath = Path::new(&args.arg_inpath);
    let outpath = Path::new(&args.arg_outpath);
    let no_thumbs = args.flag_no_thumbs;

    walk_filetree(&inpath, &outpath, no_thumbs);
}