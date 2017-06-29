# thumbrs

Photo thumbnailer and metadata extractor written in rust

[![Build Status](https://travis-ci.org/thorbenk/thumbrs.svg?branch=master)](https://travis-ci.org/thorbenk/thumbrs)

[screencast](https://raw.githubusercontent.com/thorbenk/thumbrs/master/thumbrs.gif)

## Usage

```bash
thumbrs <inpath> <outpath>
```

This will walk the given input path recursively, finding all photos. For each
photo, `thumbrs` will
- extract EXIV metadata (like file size, orientation,
  rating, camera model). The metadata of all photos within a directory are
  aggregated into a `_<dirname>.json` file
- generate thumbnails in various sizes (in parallel)
The input's directory structure is mirrored in `<outpath>`.

## Building

On Ubuntu 17.04:

```bash
sudo apt install dh-autoreconf nasm # for building mozjpeg-sys
sudo apt install libgexiv2-dev

git clone https://github.com/thorbenk/thumbrs.git && cd thumbrs
cargo build --release

