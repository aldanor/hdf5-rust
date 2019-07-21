# `hdf5`

[![Build Status](https://img.shields.io/travis/aldanor/hdf5-rust.svg)](https://travis-ci.org/aldanor/hdf5-rust) [![Appveyor Build Status](https://img.shields.io/appveyor/ci/aldanor/hdf5-rust.svg)](https://ci.appveyor.com/project/aldanor/hdf5-rust)

[Documentation](https://aldanor.github.io/hdf5-rust)
[Changelog](https://github.com/aldanor/hdf5-rust/blob/master/CHANGELOG.md)

The `hdf5` crate (previously known as `hdf5-rs`) provides thread-safe Rust bindings and 
high-level wrappers for the HDF5 library API. Some of the features include:

- Thread-safety with non-threadsafe libhdf5 builds guaranteed via reentrant mutexes.
- Native representation of most HDF5 types, including variable-length strings and arrays.
- Derive-macro for automatic mapping of user structs and enums to HDF5 types.
- Multi-dimensional array reading/writing interface via `ndarray`.

Direct low-level bindings are also available and are provided in the `hdf5-sys` crate.

Requires HDF5 library of version 1.8.4 or later.

## Example

```rust
#[derive(hdf5::H5Type, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Color {
    RED = 1,
    GREEN = 2,
    BLUE = 3,
}

#[derive(hdf5::H5Type, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Pixel {
    xy: (i64, i64),
    color: Color,
}

fn main() -> hdf5::Result<()> {
    use self::Color::*;
    use ndarray::{arr1, arr2};

    // so that libhdf5 doesn't print errors to stdout
    let _e = hdf5::silence_errors();

    {
        // write
        let file = hdf5::File::create("pixels.h5")?;
        let colors = file.new_dataset::<Color>().create("colors", 2)?;
        colors.write(&[RED, BLUE])?;
        let group = file.create_group("dir")?;
        let pixels = group.new_dataset::<Pixel>().create("pixels", (2, 2))?;
        pixels.write(&arr2(&[
            [Pixel { xy: (1, 2), color: RED }, Pixel { xy: (3, 4), color: BLUE }],
            [Pixel { xy: (5, 6), color: GREEN }, Pixel { xy: (7, 8), color: RED }],
        ]))?;
    }
    {
        // read
        let file = hdf5::File::open("pixels.h5")?;
        let colors = file.dataset("colors")?;
        assert_eq!(colors.read_1d::<Color>()?, arr1(&[RED, BLUE]));
        let pixels = file.dataset("dir/pixels")?;
        assert_eq!(
            pixels.read_raw::<Pixel>()?,
            vec![
                Pixel { xy: (1, 2), color: RED },
                Pixel { xy: (3, 4), color: BLUE },
                Pixel { xy: (5, 6), color: GREEN },
                Pixel { xy: (7, 8), color: RED },
            ]
        );
    }
    Ok(())
}
```

## Compatibility

### Platforms

`hdf5` crate is known to run on these platforms: Linux, macOS, Windows (tested on Travis 
CI and AppVeyor, HDF5 1.8 and 1.10, system installations and conda environments).

### Rust

`hdf5` crate is tested continuously for all three official release channels, and requires 
a modern Rust compiler (e.g. of version 1.31 or later).

### HDF5

Required HDF5 version is 1.8.4 or newer. The library doesn't have to be built with
threadsafe option enabled.

## Building

### HDF5 version

Build scripts for both `hdf5-sys` and `hdf5` crates check the actual version of the
HDF5 library that they are being linked against, and some functionality may be conditionally
enabled or disabled at compile time. While this allows supporting multiple versions of HDF5
in a single codebase, this is something the library user should be aware of in case they
choose to use the low level FFI bindings.

### Environment variables

If `HDF5_DIR` is set, the build script will look there (and nowhere else) for HDF5
headers and binaries (i.e., it will look for headers under `$HDF5_DIR/include`).

If `HDF5_VERSION` is set, the build script will check that the library version matches
the specified version string; in some cases it may also be used by the build script to
help locating the library (e.g. when both 1.8 and 1.10 are installed via Homebrew on macOS).

### conda

It is possible to link against `hdf5` conda package; a few notes and tips:

- Point `HDF5_DIR` to conda environment root.
- The build script knows about conda environment layout specifics and will adjust
  paths accordingly (e.g. `Library` subfolder in Windows environments).
- On Windows, environment's `bin` folder must be in `PATH` (or the environment can
  be activated prior to running cargo).
- On Linux / macOS, it is recommended to set rpath, e.g. by setting
  `RUSTFLAGS="-C link-args=-Wl,-rpath,$HDF5_DIR/lib"`.
- For old versions of HDF5 conda packages on macOS, it may also be necessary to set
  `DYLD_FALLBACK_LIBRARY_PATH="$HDF5_DIR/lib"`.

### Linux

The build script will attempt to use pkg-config first, which will likely work out without
further tweaking for the more recent versions of HDF5. The build script will then also look 
in some standard locations where HDF5 can be found after being apt-installed on Ubuntu.

### macOS

On macOS, the build script will attempt to locate HDF5 via Homebrew if it's available.
If both 1.8 and 1.10 are installed and available, the default (1.10) will be used 
unless `HDF5_VERSION` is set.

### Windows

`hdf5` crate fully supports MSVC toolchain, which allows using the
[official releases](https://www.hdfgroup.org/downloads/index.html) of
HDF5 and is generally the recommended way to go. That being said, previous experiments have 
shown that all tests pass on the `gnu` target as well, one just needs to be careful with 
building the HDF5 binary itself and configuring the build environment.

Few things to note when building on Windows:

- `hdf5.dll` should be available in the search path at build time and runtime (both `gnu` and `msvc`).
  This normally requires adding the `bin` folder of HDF5 installation to `PATH`. If using an official
  HDF5 release (`msvc` only), this will typically be done automatically by the installer.
- `msvc`: installed Visual Studio version should match the HDF5 binary (2013 or 2015). Note that it
  is not necessary to run `vcvars` scripts; Rust build system will take care of that.
- When building for either target, make sure that there are no conflicts in the search path (e.g.,
  some binaries from MinGW toolchain may shadow MSVS executables or vice versa).
- The recommended platform for `gnu` target is [TDM distribution](http://tdm-gcc.tdragon.net/) of
  MinGW-GCC as it contains bintools for both 32-bit and 64-bit.
- The recommended setup for `msvc` target is VS2015 x64 since that matches CI build configuration,
  however VS2013 and x86 should work equally well.

## License

`hdf5` crate is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details.
