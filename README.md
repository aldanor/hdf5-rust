# hdf5-rs

[![Build Status](https://img.shields.io/travis/aldanor/hdf5-rs.svg)](https://travis-ci.org/aldanor/hdf5-rs) [![Appveyor Build Status](https://img.shields.io/appveyor/ci/aldanor/hdf5-rs.svg)](https://ci.appveyor.com/project/aldanor/hdf5-rs)

[Documentation](https://docs.rs/crate/hdf5-rs)
[Changelog](https://github.com/aldanor/hdf5-rs/blob/master/CHANGELOG.md)

Thread-safe Rust bindings and high-level wrappers for the HDF5 library API.

Requires HDF5 library of version 1.8.4 or later.

## Example

```rust
#[derive(h5::H5Type, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Color {
    RED = 1,
    GREEN = 2,
    BLUE = 3,
}

#[derive(h5::H5Type, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Pixel {
    xy: (i64, i64),
    color: Color,
}

fn main() -> h5::Result<()> {
    use self::Color::*;
    use ndarray::{arr1, arr2};

    {
        // write
        let file = h5::File::open("pixels.h5", "w")?;
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
        let file = h5::File::open("pixels.h5", "r")?;
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

`hdf5-rs` is known to run on these platforms:

- Linux (tested on Travis CI, HDF5 v1.8.4)
- OS X (tested on Travis CI, HDF5 v1.8.16)
- Windows (tested on AppVeyor, MSVC target, HDF5 v1.8.16, VS2015 x64)

### Rust

`hdf5-rs` is tested for all three official release channels, and requires Rust compiler
of version 1.13 or newer.

## Building

### HDF5 version

Build scripts for both `libhdf5-sys` and `hdf5-rs` crates check the actual version of the
HDF5 library that they are being linked against, and some functionality may be conditionally
enabled or disabled at compile time. While this allows supporting multiple versions of HDF5
in a single codebase, this is something the library user should be aware of in case they
choose to use the low level FFI bindings.

### Linux, OS X

The build script of `libhdf5-lib` crate will try to use `pkg-config` if it's available
to deduce HDF5 library location. This is sufficient for most standard setups.

There are also two environment variables that may be of use if the library location and/or name
is unconventional:

- `HDF5_LIBDIR` – added to library search path during the build step
- `HDF5_LIBNAME` – library name (defaults to `hdf5`)

Note that `cargo clean` is requred before rebuilding if any of those variables are changed.

### Windows

`hdf5-rs` fully supports MSVC toolchain, which allows using the
[official releases](https://www.hdfgroup.org/downloads/index.html) of
HDF5 and is generally the recommended way to go. That being said, previous experiments have shown
that all tests pass on the `gnu` target as well, one just needs to be careful with building the
HDF5 binary itself and configuring the build environment.

Few things to note when building on Windows:

- `hdf5.dll` should be available in the search path at build time and runtime (both `gnu` and `msvc`).
  This normally requires adding the `bin` folder of HDF5 installation to `PATH`. If using an official
  HDF5 release (`msvc` only), this will be done automatically by the installer.
- If `HDF5_LIBDIR` or `HDF5_LIBNAME` change, `cargo clean` is required before rebuilding.
- `msvc`: installed Visual Studio version should match the HDF5 binary (2013 or 2015). Note that it
  is not necessary to run `vcvars` scripts; Rust build system will take care of that.
- In most cases, it is not necessary to manually set `HDF5_LIBDIR` as it would be inferred from the
  search path (both `gnu` and `msvc`). This also implies that the official releases should work
  out of the box.
- When building for either target, make sure that there are no conflicts in the search path (e.g.,
  some binaries from MinGW toolchain may shadow MSVS executables or vice versa).
- The recommended platform for `gnu` target is [TDM distribution](http://tdm-gcc.tdragon.net/) of
  MinGW-GCC as it contains bintools for both 32-bit and 64-bit.
- The recommended setup for `msvc` target is VS2015 x64 since that matches CI build configuration,
  however VS2013 and x86 should work equally well.

## License

`hdf5-rs` is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details.
