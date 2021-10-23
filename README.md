# hdf5-rust

HDF5 for Rust.

[![Build](https://github.com/aldanor/hdf5-rust/workflows/CI/badge.svg)](https://github.com/aldanor/hdf5-rust/actions?query=branch%3Amaster)
[![Latest Version](https://img.shields.io/crates/v/hdf5.svg)](https://crates.io/crates/hdf5)
[![Documentation](https://docs.rs/hdf5/badge.svg)](https://docs.rs/hdf5)
[![Changelog](https://img.shields.io/github/v/release/aldanor/hdf5-rust)](https://github.com/aldanor/hdf5-rust/blob/master/CHANGELOG.md)
![hdf5: rustc 1.51+](https://img.shields.io/badge/hdf5-rustc_1.51+-lightblue.svg)
[![Total Lines](https://tokei.rs/b1/github/aldanor/hdf5-rust)](https://github.com/aldanor/hdf5-rust)
[![Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

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
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;
use hdf5::{File, H5Type, Result};
use ndarray::{arr2, s};

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(u8)]
pub enum Color {
    R = 1,
    G = 2,
    B = 3,
}

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
pub struct Pixel {
    xy: (i64, i64),
    color: Color,
}

impl Pixel {
    pub fn new(x: i64, y: i64, color: Color) -> Self {
        Self { xy: (x, y), color }
    }
}

fn write_hdf5() -> Result<()> {
    use Color::*;
    let file = File::create("pixels.h5")?; // open for writing
    let group = file.create_group("dir")?; // create a group
    #[cfg(feature = "blosc")]
    blosc_set_nthreads(2); // set number of blosc threads
    let builder = group.new_dataset_builder();
    #[cfg(feature = "blosc")]
    let builder = builder.blosc_zstd(9, true); // zstd + shuffle
    let ds = builder
        .with_data(&arr2(&[
            // write a 2-D array of data
            [Pixel::new(1, 2, R), Pixel::new(2, 3, B)],
            [Pixel::new(3, 4, G), Pixel::new(4, 5, R)],
            [Pixel::new(5, 6, B), Pixel::new(6, 7, G)],
        ]))
        // finalize and write the dataset
        .create("pixels")?;
    // create an attr with fixed shape but don't write the data
    let attr = ds.new_attr::<Color>().shape([3]).create("colors")?;
    // write the attr data
    attr.write(&[R, G, B])?;
    Ok(())
}

fn read_hdf5() -> Result<()> {
    use Color::*;
    let file = File::open("pixels.h5")?; // open for reading
    let ds = file.dataset("dir/pixels")?; // open the dataset
    assert_eq!(
        // read a slice of the 2-D dataset and verify it
        ds.read_slice::<Pixel, _, _>(s![1.., ..])?,
        arr2(&[
            [Pixel::new(3, 4, G), Pixel::new(4, 5, R)],
            [Pixel::new(5, 6, B), Pixel::new(6, 7, G)],
        ])
    );
    let attr = ds.attr("colors")?; // open the attribute
    assert_eq!(attr.read_1d::<Color>()?.as_slice().unwrap(), &[R, G, B]);
    Ok(())
}

fn main() -> Result<()> {
    write_hdf5()?;
    read_hdf5()?;
    Ok(())
}
```

## Compatibility

### Platforms

`hdf5` crate is known to run on these platforms: Linux, macOS, Windows (tested on:
Ubuntu 16.04, 18.04, and 20.04; Windows Server 2019 with both MSVC and GNU 
toolchains; macOS Catalina).

### Rust

`hdf5` crate is tested continuously for all three official release channels, and
requires a reasonably recent Rust compiler (e.g. of version 1.51 or newer).

### HDF5

Required HDF5 version is 1.8.4 or newer. The library doesn't have to be built with
threadsafe option enabled in order to make the user code threadsafe.

Various HDF5 installation options are supported and tested: via package managers
like homebrew and apt; system-wide installations on Windows; conda installations 
from both the official channels and conda-forge. On Linux and macOS, both OpenMPI 
and MPICH parallel builds are supported and tested. 

The HDF5 C library can also be built from source and linked in statically by 
enabling `hdf5-sys/static` feature (CMake required).

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
